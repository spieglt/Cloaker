#pragma once


// CPasswordConfirmDlg dialog

class CPasswordConfirmDlg : public CDialogEx
{
	DECLARE_DYNAMIC(CPasswordConfirmDlg)

public:
	CPasswordConfirmDlg(CWnd* pParent = NULL);   // standard constructor
	virtual ~CPasswordConfirmDlg();


// Dialog Data
#ifdef AFX_DESIGN_TIME
	enum { IDD = IDD_PASSWORD_CONFIRM };
#endif

protected:
	virtual void DoDataExchange(CDataExchange* pDX);    // DDX/DDV support

	DECLARE_MESSAGE_MAP()
public:
	afx_msg
	BOOL OnInitDialog();
	BOOL DestroyWindow();
	void OnBnClickedOk();
	CString m_password;
};
