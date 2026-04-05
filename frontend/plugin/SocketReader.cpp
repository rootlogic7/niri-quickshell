#include "SocketReader.h"
#include "shell_state_generated.h"
#include "client_command_generated.h"
#include <QDebug>
#include <QVariantMap>
#include <QtEndian>

using namespace NiriShell;

SocketReader::SocketReader(QObject *parent) : QObject(parent), m_socket(new QLocalSocket(this)), m_reconnectTimer(new QTimer(this)) {
    // Socket Events
    connect(m_socket, &QLocalSocket::readyRead, this, &SocketReader::onReadyRead);
    connect(m_socket, &QLocalSocket::connected, this, &SocketReader::onConnected);
    connect(m_socket, &QLocalSocket::disconnected, this, &SocketReader::onDisconnected);
    connect(m_socket, &QLocalSocket::errorOccurred, this, &SocketReader::onError);

    // Reconnect-Timer konfigurieren (Alle 2000 Millisekunden / 2 Sekunden)
    m_reconnectTimer->setInterval(2000);
    connect(m_reconnectTimer, &QTimer::timeout, this, &SocketReader::tryConnect);

    // Initiale Verbindung direkt versuchen
    tryConnect();
}

QVariantList SocketReader::workspaces() const { return m_workspaces; }
QString SocketReader::activeWindowTitle() const { return m_activeWindowTitle; }
int SocketReader::batteryPercent() const { return m_batteryPercent; }
int SocketReader::audioVolume() const { return m_audioVolume; }
bool SocketReader::audioMuted() const { return m_audioMuted; }
QString SocketReader::networkName() const { return m_networkName; }

void SocketReader::tryConnect() {
    if (m_socket->state() == QLocalSocket::UnconnectedState) {
        // Wir probieren es im Hintergrund immer wieder...
        m_socket->connectToServer("/tmp/niri-quickshell.sock");
    }
}

void SocketReader::onConnected() {
    qDebug() << "✅ Erfolgreich mit Rust-Backend verbunden!";
    m_reconnectTimer->stop(); // Wir sind drin, Timer aus!
}

void SocketReader::onDisconnected() {
    qWarning() << "⚠️ Verbindung zum Backend verloren. Starte Auto-Reconnect...";
    m_buffer.clear(); // Wichtig: Alten Müll löschen, falls das Backend im halben Paket abgestürzt ist
    m_reconnectTimer->start();
}

void SocketReader::onError(QLocalSocket::LocalSocketError socketError) {
    // Fehler bedeutet meistens "Connection Refused" (Backend aus). 
    // Wir ignorieren den Fehler und lassen den Timer (falls noch nicht an) starten.
    if (!m_reconnectTimer->isActive()) {
        m_reconnectTimer->start();
    }
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

        // --- Aktiven Fenster-Titel verarbeiten ---
        auto title_fb = shellState->active_window_title();
        QString newTitle = title_fb ? QString::fromStdString(title_fb->str()) : "";
        if (m_activeWindowTitle != newTitle) {
            m_activeWindowTitle = newTitle;
            emit activeWindowTitleChanged();
        }

        // --- Akku verarbeiten ---
        int newBat = shellState->battery_percent();
        if (m_batteryPercent != newBat) {
            m_batteryPercent = newBat;
            emit batteryPercentChanged();
        }

        // --- Audio verarbeiten ---
        int newVol = shellState->audio_volume();
        if (m_audioVolume != newVol) {
            m_audioVolume = newVol;
            emit audioVolumeChanged();
        }

        bool newMuted = shellState->audio_muted();
        if (m_audioMuted != newMuted) {
            m_audioMuted = newMuted;
            emit audioMutedChanged();
        }

        // --- Netzwerk verarbeiten ---
        auto net_fb = shellState->network_name();
        QString newNet = net_fb ? QString::fromStdString(net_fb->str()) : "Offline";
        if (m_networkName != newNet) {
            m_networkName = newNet;
            emit networkNameChanged();
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

void SocketReader::launchMenu() {
    flatbuffers::FlatBufferBuilder builder;
    // Die Aktion heißt jetzt "launch_menu", der Integer-Wert (0) ist ein Dummy, da wir ihn nicht brauchen
    auto action = builder.CreateString("launch_menu");
    auto cmd = NiriShell::CreateClientCommand(builder, action, 0); 
    builder.Finish(cmd);
    m_socket->write(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_socket->flush();
}

void SocketReader::toggleAudioMute() {
    flatbuffers::FlatBufferBuilder builder;
    auto action = builder.CreateString("toggle_audio_mute");
    auto cmd = NiriShell::CreateClientCommand(builder, action, 0);
    builder.Finish(cmd);
    m_socket->write(reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize());
    m_socket->flush();
}
