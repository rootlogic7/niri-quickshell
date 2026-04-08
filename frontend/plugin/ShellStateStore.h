#pragma once
#include <QObject>
#include <QVariantList>
#include <QString>
#include <QtQml/qqmlregistration.h>
#include "IpcClient.h"

class ShellStateStore : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QVariantList workspaces READ workspaces NOTIFY workspacesChanged)
    Q_PROPERTY(QString activeWindowTitle READ activeWindowTitle NOTIFY activeWindowTitleChanged)
    Q_PROPERTY(int batteryPercent READ batteryPercent NOTIFY batteryPercentChanged)
    Q_PROPERTY(int audioVolume READ audioVolume NOTIFY audioVolumeChanged)
    Q_PROPERTY(bool audioMuted READ audioMuted NOTIFY audioMutedChanged)
    Q_PROPERTY(QString networkName READ networkName NOTIFY networkNameChanged)
    Q_PROPERTY(int toggleCcSignal READ toggleCcSignal NOTIFY toggleCcSignalChanged)

public:
    explicit ShellStateStore(QObject *parent = nullptr);

    QVariantList workspaces() const;
    QString activeWindowTitle() const;
    int batteryPercent() const;
    int audioVolume() const;
    bool audioMuted() const;
    QString networkName() const;
    int toggleCcSignal() const;

    Q_INVOKABLE void focusWorkspace(int id);
    Q_INVOKABLE void launchMenu();
    Q_INVOKABLE void toggleAudioMute();

signals:
    void workspacesChanged();
    void activeWindowTitleChanged();
    void batteryPercentChanged();
    void audioVolumeChanged();
    void audioMutedChanged();
    void networkNameChanged();
    void toggleCcSignalChanged();

private slots:
    void processPacket(const QByteArray &packet);

private:
    IpcClient *m_ipcClient; // Instanz des Postboten

    QVariantList m_workspaces;
    QString m_activeWindowTitle;
    int m_batteryPercent = 100;
    int m_audioVolume = 0;
    bool m_audioMuted = false;
    QString m_networkName = "Offline";
    int m_toggleCcSignal = 0;
};
