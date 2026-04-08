#include "IpcClient.h"
#include <QStandardPaths>
#include <QDir>
#include <QDebug>
#include <QtEndian>

IpcClient::IpcClient(QObject *parent)
    : QObject(parent), m_socket(new QLocalSocket(this)), m_reconnectTimer(new QTimer(this)) {

    connect(m_socket, &QLocalSocket::readyRead, this, &IpcClient::onReadyRead);
    connect(m_socket, &QLocalSocket::connected, this, &IpcClient::onConnected);
    connect(m_socket, &QLocalSocket::disconnected, this, &IpcClient::onDisconnected);

    // Ignoriere Error-Signal, der Timer kümmert sich um den Reconnect
    connect(m_socket, &QLocalSocket::errorOccurred, this, [this]() {
        if (!m_reconnectTimer->isActive()) m_reconnectTimer->start();
    });

    m_reconnectTimer->setInterval(2000);
    connect(m_reconnectTimer, &QTimer::timeout, this, &IpcClient::tryConnect);

    tryConnect();
}

void IpcClient::tryConnect() {
    if (m_socket->state() == QLocalSocket::UnconnectedState) {
        QString runtimePath = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
        QString socketPath = QDir(runtimePath).filePath("niri-quickshell/ipc.sock");
        m_socket->connectToServer(socketPath);
    }
}

void IpcClient::onConnected() {
    qDebug() << "✅ IpcClient: Erfolgreich mit Rust-Backend verbunden!";
    m_reconnectTimer->stop();
}

void IpcClient::onDisconnected() {
    qWarning() << "⚠️ IpcClient: Verbindung verloren. Starte Auto-Reconnect...";
    m_buffer.clear();
    m_reconnectTimer->start();
}

void IpcClient::onReadyRead() {
    m_buffer.append(m_socket->readAll());

    // Puffer-Logik: Schneidet exakt die fertigen FlatBuffer-Pakete aus dem Stream
    while (m_buffer.size() >= 4) {
        uint32_t size = qFromLittleEndian<uint32_t>(m_buffer.constData());

        if (m_buffer.size() < size + 4) {
            break; // Paket noch nicht komplett runtergeladen
        }

        QByteArray packet = m_buffer.left(size + 4);
        m_buffer.remove(0, size + 4);

        // Fertiges Paket ans Gehirn (ShellStateStore) weiterleiten!
        emit messageReceived(packet);
    }
}

void IpcClient::sendCommand(const QByteArray &data) {
    if (m_socket->state() == QLocalSocket::ConnectedState) {
        m_socket->write(data);
        m_socket->flush();
    }
}
