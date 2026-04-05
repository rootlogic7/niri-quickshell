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
    Q_PROPERTY(int audioVolume READ audioVolume NOTIFY audioVolumeChanged) // NEU
    Q_PROPERTY(bool audioMuted READ audioMuted NOTIFY audioMutedChanged)
    Q_PROPERTY(QString networkName READ networkName NOTIFY networkNameChanged)

public:
    explicit SocketReader(QObject *parent = nullptr);
    QVariantList workspaces() const;
    int batteryPercent() const;
    QString activeWindowTitle() const;
    Q_INVOKABLE void focusWorkspace(int id);
    Q_INVOKABLE void launchMenu();
    int audioVolume() const;
    bool audioMuted() const;
    QString networkName() const;

signals:
    void workspacesChanged();
    void batteryPercentChanged();
    void activeWindowTitleChanged();
    void audioVolumeChanged();
    void audioMutedChanged();
    void networkNameChanged();

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
    int m_audioVolume = 0;
    bool m_audioMuted = false;
    QString m_networkName = "Offline";
};
