// NEU: Wir müssen Theme und ThemeArgs importieren
use crate::shell_state_generated::niri_shell::{
    ShellState, ShellStateArgs, Workspace, WorkspaceArgs, Theme, ThemeArgs
};
use flatbuffers::FlatBufferBuilder;
use tokio::io::AsyncWriteExt;
use crate::modules::{niri, audio, network, battery};

pub async fn build_and_send<W>(
    tx: &mut W,
    dbus_conn: &zbus::Connection,
    cc_counter: u8,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: AsyncWriteExt + Unpin,
{
    // 1. Daten von allen Modulen asynchron & parallel abfragen
    let (mut workspaces_data, active_title, (vol, muted)) = tokio::join!(
        niri::fetch_workspaces(),
        niri::fetch_active_window_title(),
        audio::get_audio_state()
    );

    let mut builder = FlatBufferBuilder::new();

    // -- Workspaces verarbeiten --
    workspaces_data.sort_by_key(|ws| ws.idx);
    let mut ws_offsets = Vec::new();
    for ws in workspaces_data {
        let name_str = ws.name.unwrap_or_else(|| ws.idx.to_string());
        let name_fb = builder.create_string(&name_str);
        ws_offsets.push(Workspace::create(&mut builder, &WorkspaceArgs {
            id: ws.idx as _, 
            name: Some(name_fb), 
            is_active: ws.is_active,
        }));
    }
    let workspaces_vec = builder.create_vector(&ws_offsets);
    
    // -- Einzelne Werte verarbeiten --
    let title_fb = active_title.as_ref().map(|t| builder.create_string(t));
    
    let net_name = network::get_network_name(dbus_conn).await;
    let net_name_fb = builder.create_string(&net_name);

    // ==========================================
    // LIVE-THEME LADEN
    // ==========================================
    let current_theme = crate::modules::theme::get_theme();

    let bg_color = builder.create_string(&current_theme.bg_color);
    let fg_color = builder.create_string(&current_theme.fg_color);
    let accent_color = builder.create_string(&current_theme.accent_color);

    let theme_offset = Theme::create(&mut builder, &ThemeArgs {
        bg_color: Some(bg_color),
        fg_color: Some(fg_color),
        accent_color: Some(accent_color),
    });

    // ==========================================
    // 2. NEU: Verfügbare Themes sammeln
    // ==========================================
    let available_themes_list = crate::modules::theme::get_available_themes();
    
    let mut theme_name_offsets = Vec::new();
    for name in available_themes_list {
        theme_name_offsets.push(builder.create_string(&name));
    }
    let available_themes_vec = builder.create_vector(&theme_name_offsets);

    // -- Finales ShellState Objekt bauen --
    let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
        workspaces: Some(workspaces_vec),
        battery_percent: battery::get_battery_percent(),
        active_window_title: title_fb,
        audio_volume: vol,
        audio_muted: muted,
        network_name: Some(net_name_fb),
        toggle_cc_signal: cc_counter,
        theme: Some(theme_offset),
        available_themes: Some(available_themes_vec),
    });

    builder.finish_size_prefixed(shell_state, None);
    
    // -- Über den Socket versenden --
    tx.write_all(builder.finished_data()).await?;
    Ok(())
}
