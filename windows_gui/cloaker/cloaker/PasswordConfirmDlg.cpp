// PasswordConfirmDlg.cpp : implementation file
//

#include "stdafx.h"
#include "cloaker.h"
#include "PasswordConfirmDlg.h"
#include "afxdialogex.h"


// CPasswordConfirmDlg dialog

IMPLEMENT_DYNAMIC(CPasswordConfirmDlg, CDialogEx)

CPasswordConfirmDlg::CPasswordConfirmDlg(CWnd* pParent /*=NULL*/)
	: CDialogEx(IDD_PASSWORD_CONFIRM, pParent)
{

}

CPasswordConfirmDlg::~CPasswordConfirmDlg()
{
}

void CPasswordConfirmDlg::DoDataExchange(CDataExchange* pDX)
{
	CDialogEx::DoDataExchange(pDX);
}


BEGIN_MESSAGE_MAP(CPasswordConfirmDlg, CDialogEx)
	ON_BN_CLICKED(IDC_OK, &CPasswordConfirmDlg::OnBnClickedOk)
END_MESSAGE_MAP()


// CPasswordConfirmDlg message handlers


BOOL CPasswordConfirmDlg::OnInitDialog()
{
	CDialog::OnInitDialog();

	SetDlgItemText(IDC_PASSWORD, m_password);
	return TRUE;
}

BOOL CPasswordConfirmDlg::DestroyWindow()
{
	GetDlgItemText(IDC_PASSWORD, m_password);
	return CDialog::DestroyWindow();
}

void CPasswordConfirmDlg::OnBnClickedOk()
{
	CDialog::OnOK();
}
