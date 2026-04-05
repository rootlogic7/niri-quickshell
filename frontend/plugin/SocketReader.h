// frontend/plugin/SocketReader.h
#pragma once

#include <QObject>
#include <QLocalSocket>
#include <QVariantList>
#include <QByteArray>
#include <QTimer>
#include <QtQml/qqml.h>

class SocketReader : public QObject {
    Q_OBJECT
    QML_ELEMENT
    // Dieses Property ist später in QML als "workspaces" abrufbar
    Q_PROPERTY(QVariantList workspaces READ workspaces NOTIFY workspacesChanged)
    Q_PROPERTY(int batteryPercent READ batteryPercent NOTIFY batteryPercentChanged)
    Q_PROPERTY(QString activeWindowTitle READ activeWindowTitle NOTIFY activeWindowTitleChanged)

public:
    explicit SocketReader(QObject *parent = nullptr);
    QVariantList workspaces() const;
    int batteryPercent() const;
    Q_INVOKABLE void focusWorkspace(int id);
    QString activeWindowTitle() const;

signals:
    void workspacesChanged();
    void batteryPercentChanged();
    void activeWindowTitleChanged();

private slots:
    void onReadyRead();
    void onConnected();
    void onDisconnected();
    void onError(QLocalSocket::LocalSocketError socketError);
    void tryConnect();

private:
    QLocalSocket *m_socket;
    QTimer *m_reconnectTimer;
    QVariantList m_workspaces;
    int m_batteryPercent = 0;
    QByteArray m_buffer;
    QString m_activeWindowTitle;
};
