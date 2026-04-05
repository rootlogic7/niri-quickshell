// frontend/plugin/SocketReader.cpp
#include "SocketReader.h"
#include "shell_state_generated.h" // Unser generierter Vertrag
#include <QDebug>
#include <QVariantMap>

using namespace NiriShell; // Aus dem .fbs Schema

SocketReader::SocketReader(QObject *parent) : QObject(parent), m_socket(new QLocalSocket(this)) {
    connect(m_socket, &QLocalSocket::readyRead, this, &SocketReader::onReadyRead);
    connect(m_socket, &QLocalSocket::connected, this, &SocketReader::onConnected);
    connect(m_socket, &QLocalSocket::errorOccurred, this, &SocketReader::onError);

    // Verbinde zum Rust-Daemon
    qDebug() << "Versuche mit Rust-Backend zu verbinden...";
    m_socket->connectToServer("/tmp/niri-quickshell.sock");
}

QVariantList SocketReader::workspaces() const {
    return m_workspaces;
}

void SocketReader::onConnected() {
    qDebug() << "Erfolgreich mit Rust-Backend verbunden!";
}

void SocketReader::onError(QLocalSocket::LocalSocketError socketError) {
    qWarning() << "Socket-Fehler:" << m_socket->errorString();
}

void SocketReader::onReadyRead() {
    // 1. Lies alle rohen Bytes aus dem Socket
    QByteArray data = m_socket->readAll();

    // 2. Sicherheitscheck: Ist das Paket valide?
    flatbuffers::Verifier verifier(reinterpret_cast<const uint8_t*>(data.constData()), data.size());
    if (!VerifyShellStateBuffer(verifier)) {
        qWarning() << "Fehlerhaftes FlatBuffer-Paket empfangen!";
        return;
    }

    // 3. Zero-Copy: Schablone über den Speicher legen
    auto shellState = GetShellState(data.constData());
    auto workspaces_fb = shellState->workspaces();

    if (!workspaces_fb) return;

    QVariantList newWorkspaces;

    // 4. Daten für QML aufbereiten
    for (uint32_t i = 0; i < workspaces_fb->size(); i++) {
        auto ws = workspaces_fb->Get(i);
        QVariantMap map;
        map["id"] = QVariant::fromValue(ws->id());
        map["name"] = QString::fromStdString(ws->name()->str());
        map["is_active"] = ws->is_active();
        newWorkspaces.append(map);
    }

    // 5. QML benachrichtigen, wenn sich die Daten geändert haben
    if (m_workspaces != newWorkspaces) {
        m_workspaces = newWorkspaces;
        emit workspacesChanged();
    }
}
