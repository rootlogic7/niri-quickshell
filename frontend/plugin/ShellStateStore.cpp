#include "ShellStateStore.h"
#include "shell_state_generated.h"
#include "client_command_generated.h"
#include <QVariantMap>
#include <QDebug>

ShellStateStore::ShellStateStore(QObject *parent) : QObject(parent), m_ipcClient(new IpcClient(this)) {
    // Verbinde den Postboten mit unserem Verarbeiter
    connect(m_ipcClient, &IpcClient::messageReceived, this, &ShellStateStore::processPacket);
}

// ... [Getter-Methoden hier einfügen, exakt wie im alten SocketReader] ...
QVariantList ShellStateStore::workspaces() const { return m_workspaces; }
QString ShellStateStore::activeWindowTitle() const { return m_activeWindowTitle; }
int ShellStateStore::batteryPercent() const { return m_batteryPercent; }
int ShellStateStore::audioVolume() const { return m_audioVolume; }
bool ShellStateStore::audioMuted() const { return m_audioMuted; }
QString ShellStateStore::networkName() const { return m_networkName; }
int ShellStateStore::toggleCcSignal() const { return m_toggleCcSignal; }

void ShellStateStore::processPacket(const QByteArray &packet) {
    flatbuffers::Verifier verifier(reinterpret_cast<const uint8_t*>(packet.constData()), packet.size());
    if (!NiriShell::VerifySizePrefixedShellStateBuffer(verifier)) {
        qWarning() << "Fehlerhaftes FlatBuffer-Paket empfangen!";
        return;
    }

    auto shellState = NiriShell::GetSizePrefixedShellState(packet.constData());

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

    // --- Werte verarbeiten (Wie gehabt) ---
    auto title_fb = shellState->active_window_title();
    QString newTitle = title_fb ? QString::fromStdString(title_fb->str()) : "";
    if (m_activeWindowTitle != newTitle) {
        m_activeWindowTitle = newTitle;
        emit activeWindowTitleChanged();
    }

    int newBat = shellState->battery_percent();
    if (m_batteryPercent != newBat) { m_batteryPercent = newBat; emit batteryPercentChanged(); }

    int newVol = shellState->audio_volume();
    if (m_audioVolume != newVol) { m_audioVolume = newVol; emit audioVolumeChanged(); }

    bool newMuted = shellState->audio_muted();
    if (m_audioMuted != newMuted) { m_audioMuted = newMuted; emit audioMutedChanged(); }

    auto net_fb = shellState->network_name();
    QString newNet = net_fb ? QString::fromStdString(net_fb->str()) : "Offline";
    if (m_networkName != newNet) { m_networkName = newNet; emit networkNameChanged(); }

    int new_cc_signal = shellState->toggle_cc_signal();
    if (m_toggleCcSignal != new_cc_signal) { m_toggleCcSignal = new_cc_signal; emit toggleCcSignalChanged(); }
}

void ShellStateStore::focusWorkspace(int id) {
    flatbuffers::FlatBufferBuilder builder;
    auto action = builder.CreateString("focus_workspace");
    auto cmd = NiriShell::CreateClientCommand(builder, action, id);
    builder.Finish(cmd);

    QByteArray data(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_ipcClient->sendCommand(data);
}

void ShellStateStore::launchMenu() {
    flatbuffers::FlatBufferBuilder builder;
    auto action = builder.CreateString("launch_menu");
    auto cmd = NiriShell::CreateClientCommand(builder, action, 0);
    builder.Finish(cmd);

    QByteArray data(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_ipcClient->sendCommand(data);
}

void ShellStateStore::toggleAudioMute() {
    flatbuffers::FlatBufferBuilder builder;
    auto action = builder.CreateString("toggle_audio_mute");
    auto cmd = NiriShell::CreateClientCommand(builder, action, 0);
    builder.Finish(cmd);

    QByteArray data(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_ipcClient->sendCommand(data);
}
