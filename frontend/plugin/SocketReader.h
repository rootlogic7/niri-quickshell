// frontend/plugin/SocketReader.h
#pragma once

#include <QObject>
#include <QLocalSocket>
#include <QVariantList>
#include <QtQml/qqml.h>

class SocketReader : public QObject {
    Q_OBJECT
    QML_ELEMENT
    // Dieses Property ist später in QML als "workspaces" abrufbar
    Q_PROPERTY(QVariantList workspaces READ workspaces NOTIFY workspacesChanged)

public:
    explicit SocketReader(QObject *parent = nullptr);
    QVariantList workspaces() const;

signals:
    void workspacesChanged();

private slots:
    void onReadyRead();
    void onConnected();
    void onError(QLocalSocket::LocalSocketError socketError);

private:
    QLocalSocket *m_socket;
    QVariantList m_workspaces;
};
