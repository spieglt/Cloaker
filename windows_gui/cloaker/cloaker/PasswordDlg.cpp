// PasswordDlg.cpp : implementation file
//

#include "stdafx.h"
#include "cloaker.h"
#include "PasswordDlg.h"
#include "afxdialogex.h"


// CPasswordDlg dialog

IMPLEMENT_DYNAMIC(CPasswordDlg, CDialogEx)

CPasswordDlg::CPasswordDlg(CWnd* pParent /*=NULL*/)
	: CDialogEx(IDD_PASSWORD, pParent)
{

}

CPasswordDlg::~CPasswordDlg()
{
}

void CPasswordDlg::DoDataExchange(CDataExchange* pDX)
{
	CDialogEx::DoDataExchange(pDX);
	//DDX_Control(pDX, IDC_PASSWORD, m_password);
}


BEGIN_MESSAGE_MAP(CPasswordDlg, CDialogEx)
	ON_BN_CLICKED(IDC_OK, &CPasswordDlg::OnBnClickedOk)
END_MESSAGE_MAP()


// CPasswordDlg message handlers

BOOL CPasswordDlg::OnInitDialog()
{
	CDialog::OnInitDialog();

	SetDlgItemText(IDC_PASSWORD, m_password);
	return TRUE;
}

BOOL CPasswordDlg::DestroyWindow()
{
	GetDlgItemText(IDC_PASSWORD, m_password);
	return CDialog::DestroyWindow();
}

void CPasswordDlg::OnBnClickedOk()
{
	CDialog::OnOK();
}
