#-------------------------------------------------
#
# Project created by QtCreator 2019-06-06T20:52:47
#
#-------------------------------------------------

QT       += core gui

greaterThan(QT_MAJOR_VERSION, 4): QT += widgets

TARGET = cloaker
TEMPLATE = app

# The following define makes your compiler emit warnings if you use
# any feature of Qt which has been marked as deprecated (the exact warnings
# depend on your compiler). Please consult the documentation of the
# deprecated API in order to know how to port your code away from it.
DEFINES += QT_DEPRECATED_WARNINGS

# You can also make your code fail to compile if you use deprecated APIs.
# In order to do so, uncomment the following line.
# You can also select to disable deprecated APIs only up to a certain version of Qt.
#DEFINES += QT_DISABLE_DEPRECATED_BEFORE=0x060000    # disables all the APIs deprecated before Qt 6.0.0

CONFIG += c++11

SOURCES += \
        main.cpp \
        mainwindow.cpp \
    droparea.cpp \
    adapter.cpp

HEADERS += \
        mainwindow.h \
    droparea.h \
    adapter.h

FORMS += \
        mainwindow.ui

# Default rules for deployment.
qnx: target.path = /tmp/$${TARGET}/bin
else: unix:!android: target.path = /opt/$${TARGET}/bin
!isEmpty(target.path): INSTALLS += target


unix: LIBS += -L$$PWD/../../gui_adapter/target/release/ -ladapter

unix: LIBS += -ldl

INCLUDEPATH += $$PWD/../../gui_adapter/target/release
DEPENDPATH += $$PWD/../../gui_adapter/target/release

unix: PRE_TARGETDEPS += $$PWD/../../gui_adapter/target/release/libadapter.a

DISTFILES +=

ICON = macCloakerLogo.icns

win32: LIBS += -L$$PWD/../../gui_adapter/target/x86_64-pc-windows-gnu/release/ -ladapter -lws2_32 -luserenv

INCLUDEPATH += $$PWD/../../gui_adapter/target/x86_64-pc-windows-gnu/release
DEPENDPATH += $$PWD/../../gui_adapter/target/x86_64-pc-windows-gnu/release

win32:!win32-g++: PRE_TARGETDEPS += $$PWD/../../gui_adapter/target/x86_64-pc-windows-gnu/release/adapter.lib
else:win32-g++: PRE_TARGETDEPS += $$PWD/../../gui_adapter/target/x86_64-pc-windows-gnu/release/libadapter.a
