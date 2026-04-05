// frontend/plugin/SocketReader.h
#pragma once

#include <QObject>
#include <QLocalSocket>
#include <QVariantList>
#include <QByteArray>
#include <QtQml/qqml.h>

class SocketReader : public QObject {
    Q_OBJECT
    QML_ELEMENT
    // Dieses Property ist später in QML als "workspaces" abrufbar
    Q_PROPERTY(QVariantList workspaces READ workspaces NOTIFY workspacesChanged)
    Q_PROPERTY(int batteryPercent READ batteryPercent NOTIFY batteryPercentChanged)

public:
    explicit SocketReader(QObject *parent = nullptr);
    QVariantList workspaces() const;
    int batteryPercent() const;
    Q_INVOKABLE void focusWorkspace(int id);

signals:
    void workspacesChanged();
    void batteryPercentChanged();

private slots:
    void onReadyRead();
    void onConnected();
    void onError(QLocalSocket::LocalSocketError socketError);

private:
    QLocalSocket *m_socket;
    QVariantList m_workspaces;
    int m_batteryPercent = 0;
    QByteArray m_buffer;
};
