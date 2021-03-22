#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>

namespace Ui {
class MainWindow;
}

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();
    void updateProgress(int);

private slots:

    void on_actionAbout_Cloaker_triggered();


private:
    Ui::MainWindow *ui;
};

extern MainWindow *gMainWindow;


#endif // MAINWINDOW_H
