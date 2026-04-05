cmake_minimum_required(VERSION 3.16)
project(NiriQuickshellPlugin LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

# Fix für Warnung 1: Qt6 Policy QTP0001 aktivieren (Standard seit Qt 6.5+)
if(POLICY QTP0001)
    cmake_policy(SET QTP0001 NEW)
endif()

find_package(Qt6 REQUIRED COMPONENTS Core Network Qml Quick)

# Fix für Warnung 2: Wir fügen OUTPUT_DIRECTORY hinzu, um die Pfad-Warnung zu beheben
qt6_add_qml_module(NiriStatePlugin
    URI NiriState
    VERSION 1.0
    OUTPUT_DIRECTORY NiriState
    SOURCES
        plugin/SocketReader.cpp
        plugin/SocketReader.h
)

# DER FIX FÜR DEN ERROR: Wir sagen dem Compiler explizit, wo unsere Header liegen!
target_include_directories(NiriStatePlugin PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/plugin
)

target_link_libraries(NiriStatePlugin PRIVATE
    Qt6::Core
    Qt6::Network
    Qt6::Qml
    Qt6::Quick
)
