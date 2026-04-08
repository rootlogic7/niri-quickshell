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
    Q_PROPERTY(int toggleCcSignal READ toggleCcSignal NOTIFY toggleCcSignalChanged)

public:
    explicit SocketReader(QObject *parent = nullptr);
    QVariantList workspaces() const;
    int batteryPercent() const;
    QString activeWindowTitle() const;
    Q_INVOKABLE void focusWorkspace(int id);
    Q_INVOKABLE void launchMenu();
    Q_INVOKABLE void toggleAudioMute();
    int audioVolume() const;
    bool audioMuted() const;
    QString networkName() const;
    int toggleCcSignal() const { return m_toggleCcSignal; }

signals:
    void workspacesChanged();
    void batteryPercentChanged();
    void activeWindowTitleChanged();
    void audioVolumeChanged();
    void audioMutedChanged();
    void networkNameChanged();
    void toggleCcSignalChanged();

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
    int m_toggleCcSignal = 0;
};
