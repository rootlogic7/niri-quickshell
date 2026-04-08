#pragma once
#include <QObject>
#include <QLocalSocket>
#include <QTimer>
#include <QByteArray>

class IpcClient : public QObject {
    Q_OBJECT
public:
    explicit IpcClient(QObject *parent = nullptr);
    void sendCommand(const QByteArray &data);

signals:
    void messageReceived(const QByteArray &packet);

private slots:
    void tryConnect();
    void onConnected();
    void onDisconnected();
    void onReadyRead();

private:
    QLocalSocket *m_socket;
    QTimer *m_reconnectTimer;
    QByteArray m_buffer;
};
