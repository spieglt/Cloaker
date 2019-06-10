
// cloakerDlg.h : header file
//

#pragma once
#include "DropTarget.h"

// CcloakerDlg dialog
class CcloakerDlg : public CDialogEx
{
// Construction
public:
	CcloakerDlg(CWnd* pParent = NULL);	// standard constructor
	~CcloakerDlg();

// Dialog Data
#ifdef AFX_DESIGN_TIME
	enum { IDD = IDD_CLOAKER_DIALOG };
#endif

	protected:
	virtual void DoDataExchange(CDataExchange* pDX);	// DDX/DDV support


// Implementation
private:
	// Child window!
	CDropTarget* m_dropTarget;

protected:
	HICON m_hIcon;

	// Generated message map functions
	virtual BOOL OnInitDialog();
	afx_msg void OnSysCommand(UINT nID, LPARAM lParam);
	afx_msg void OnPaint();
	afx_msg HCURSOR OnQueryDragIcon();
	DECLARE_MESSAGE_MAP()
public:
	afx_msg void OnBnClickedRadio1();
};
