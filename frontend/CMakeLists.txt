cmake_minimum_required(VERSION 3.16)
project(NiriQuickshell LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

# 1. Corrosion laden (Lädt das Tool herunter)
find_package(Corrosion QUIET)
if(NOT Corrosion_FOUND)
    message(STATUS "Corrosion nicht im System gefunden. Lade via FetchContent herunter...")
    include(FetchContent)
    FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.5.0
    )
    FetchContent_MakeAvailable(Corrosion)
endif()

# 2. Rust-Projekt importieren
corrosion_import_crate(MANIFEST_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../backend/Cargo.toml")

# 3. Qt-Frontend konfigurieren
find_package(Qt6 REQUIRED COMPONENTS Core Network Qml Quick)

# FIX: Die Qt-Policy wird mit 'qt_policy' NACH find_package gesetzt!
qt_policy(SET QTP0001 NEW)

qt6_add_qml_module(NiriStatePlugin
    URI NiriState
    VERSION 1.0
    OUTPUT_DIRECTORY NiriState
    SOURCES
        plugin/IpcClient.cpp
        plugin/IpcClient.h
        plugin/ShellStateStore.cpp
        plugin/ShellStateStore.h
)

target_include_directories(NiriStatePlugin PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/plugin
)

target_link_libraries(NiriStatePlugin PRIVATE
    Qt6::Core
    Qt6::Network
    Qt6::Qml
    Qt6::Quick
)

# 4. Installation
# 4.1. Das Rust-Backend installieren (/usr/bin/)
install(PROGRAMS $<TARGET_FILE:niri-quickshell-backend> 
        DESTINATION bin)

# 4.2. Das C++ QML-Plugin installieren
# MAGIE: Wir sagen dem Wrapper, dass er im gleichen Ordner ($ORIGIN) nach der Basis-Lib suchen soll!
set_target_properties(NiriStatePluginplugin PROPERTIES INSTALL_RPATH "$ORIGIN")

# Wir lassen CMake beide Bibliotheken sauber installieren (CMake repariert dabei die Pfade!)
install(TARGETS NiriStatePlugin NiriStatePluginplugin 
        DESTINATION lib/niri-quickshell/NiriState)

# Und wir kopieren die generierten QML-Textdateien hinterher
install(FILES 
        ${CMAKE_CURRENT_BINARY_DIR}/NiriState/qmldir 
        ${CMAKE_CURRENT_BINARY_DIR}/NiriState/NiriStatePlugin.qmltypes 
        DESTINATION lib/niri-quickshell/NiriState)

# 4.3. Die QML UI-Dateien installieren (/usr/share/niri-quickshell/ui/)
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/ui 
        DESTINATION share/niri-quickshell)

# 4.4. Unser neues Wrapper-Skript installieren (/usr/bin/)
install(PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/../bin/niri-quickshell 
        DESTINATION bin)
