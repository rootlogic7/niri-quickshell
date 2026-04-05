#include "SocketReader.h"
#include "shell_state_generated.h"
#include "client_command_generated.h"
#include <QDebug>
#include <QVariantMap>
#include <QtEndian> // NEU: Für sicheres Auslesen des 4-Byte Size-Prefix

using namespace NiriShell;

SocketReader::SocketReader(QObject *parent) : QObject(parent), m_socket(new QLocalSocket(this)) {
    connect(m_socket, &QLocalSocket::readyRead, this, &SocketReader::onReadyRead);
    connect(m_socket, &QLocalSocket::connected, this, &SocketReader::onConnected);
    connect(m_socket, &QLocalSocket::errorOccurred, this, &SocketReader::onError);
    m_socket->connectToServer("/tmp/niri-quickshell.sock");
}

QVariantList SocketReader::workspaces() const { return m_workspaces; }
int SocketReader::batteryPercent() const { return m_batteryPercent; }

void SocketReader::onConnected() { qDebug() << "Erfolgreich mit Rust-Backend verbunden!"; }
void SocketReader::onError(QLocalSocket::LocalSocketError socketError) {
    qWarning() << "Socket-Fehler:" << m_socket->errorString();
}

void SocketReader::onReadyRead() {
    // 1. Alles in den Puffer schieben
    m_buffer.append(m_socket->readAll());

    // 2. Solange genug Bytes für das 4-Byte Größen-Präfix da sind...
    while (m_buffer.size() >= 4) {
        // Little-Endian 32-bit Integer auslesen (FlatBuffers Standard)
        uint32_t size = qFromLittleEndian<uint32_t>(m_buffer.constData());

        // 3. Ist das gesamte Paket angekommen (Größe + 4 Bytes Präfix)?
        if (m_buffer.size() < size + 4) {
            break; // Nein, weiter warten auf den restlichen Stream!
        }

        // 4. Ein komplettes Paket ausschneiden
        QByteArray packet = m_buffer.left(size + 4);
        m_buffer.remove(0, size + 4);

        // 5. Verifizieren und Entpacken (Achtung: SizePrefixed Versionen!)
        flatbuffers::Verifier verifier(reinterpret_cast<const uint8_t*>(packet.constData()), packet.size());
        if (!VerifySizePrefixedShellStateBuffer(verifier)) {
            qWarning() << "Fehlerhaftes Size-Prefixed FlatBuffer-Paket empfangen!";
            continue;
        }

        auto shellState = GetSizePrefixedShellState(packet.constData());

        // --- Workspaces verarbeiten ---
        auto workspaces_fb = shellState->workspaces();
        if (workspaces_fb) {
            QVariantList newWorkspaces;
            for (uint32_t i = 0; i < workspaces_fb->size(); i++) {
                auto ws = workspaces_fb->Get(i);
                QVariantMap map;
                map["id"] = QVariant::fromValue(ws->id());
                map["name"] = QString::fromStdString(ws->name()->str());
                map["is_active"] = ws->is_active();
                newWorkspaces.append(map);
            }
            if (m_workspaces != newWorkspaces) {
                m_workspaces = newWorkspaces;
                emit workspacesChanged();
            }
        }

        // --- Akku verarbeiten ---
        int newBat = shellState->battery_percent();
        if (m_batteryPercent != newBat) {
            m_batteryPercent = newBat;
            emit batteryPercentChanged();
        }
    }
}

void SocketReader::focusWorkspace(int id) {
    flatbuffers::FlatBufferBuilder builder;
    auto action = builder.CreateString("focus_workspace");
    auto cmd = NiriShell::CreateClientCommand(builder, action, id);
    builder.Finish(cmd); // Commands ans Backend bleiben einfach (kein Prefix nötig)
    m_socket->write(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_socket->flush();
}
