import LandingPage from './components/LandingPage';
import Dashboard from './components/Dashboard';
import CreateAccountModal from './components/CreateAccountModal';
import AddEndpointModal from './components/AddEndpointModal';
import SetIntervallModal from './components/SetIntervallModal';
import DeleteConfirmModal from './components/DeleteConfirmModal';
import AccountSettingsModal from './components/AccountSettingsModal';
import LogModal from './components/LogModal';
import EditUrlModal from './components/EditUrlModal';

import { ThemeProvider } from './ThemeContext';
import { useAppState } from './hooks/useAppState';
import './App.css';

function App() {
  const {
    user, endpoints, mainSwitch,
    showCreateAccount, setShowCreateAccount,
    showAddEndpoint, setShowAddEndpoint,
    showSetIntervall, setShowSetIntervall,
    showDeleteConfirm, setShowDeleteConfirm,
    setDeleteIndex,
    showAccountSettings, setShowAccountSettings,
    showLog, setShowLog,
    selectedEndpoint, logEntries,
    showEditUrl, setShowEditUrl,
    editUrlValue, setEditUrlValue,
    handleLogin, handleCreateAccount, handleLogout,
    handleToggleMainSwitch, handleToggleEndpoint,
    handleAddEndpoint,
    handleSetIntervall, handleSetIntervallSubmit,
    handleRemove, confirmDelete,
    handleEditUrl, handleSaveUrl,
    handleChangePassword, handleChangeEmail, handleDeleteAccount,
    handleShowLog,
  } = useAppState();

  return (
    <ThemeProvider>
      {!user ? (
        <>
          <LandingPage onLogin={handleLogin} onCreateAccount={() => setShowCreateAccount(true)} />
          <CreateAccountModal isOpen={showCreateAccount} onClose={() => setShowCreateAccount(false)} onSubmit={handleCreateAccount} />
        </>
      ) : (
        <>
          <Dashboard
            endpoints={endpoints}
            mainSwitch={mainSwitch}
            onToggleMainSwitch={handleToggleMainSwitch}
            onRemove={handleRemove}
            onToggleEndpoint={handleToggleEndpoint}
            onSetIntervall={handleSetIntervall}
            onShowLog={handleShowLog}
            onEditUrl={handleEditUrl}
            onAddEndpoint={() => setShowAddEndpoint(true)}
            onLogout={handleLogout}
            onAccountSettings={() => setShowAccountSettings(true)}
          />

          <AddEndpointModal isOpen={showAddEndpoint} onClose={() => setShowAddEndpoint(false)} onSubmit={handleAddEndpoint} />
          <SetIntervallModal isOpen={showSetIntervall} onClose={() => setShowSetIntervall(false)} endpoint={selectedEndpoint} onSubmit={handleSetIntervallSubmit} />
          <DeleteConfirmModal isOpen={showDeleteConfirm} onClose={() => { setShowDeleteConfirm(false); setDeleteIndex(null); }} onConfirm={confirmDelete} />
          <AccountSettingsModal isOpen={showAccountSettings} onClose={() => setShowAccountSettings(false)} onChangePassword={handleChangePassword} onChangeEmail={handleChangeEmail} onDeleteAccount={handleDeleteAccount} />
          <LogModal isOpen={showLog} onClose={() => setShowLog(false)} entries={logEntries} />
          <EditUrlModal isOpen={showEditUrl} onClose={() => setShowEditUrl(false)} endpoint={selectedEndpoint} value={editUrlValue} onChange={setEditUrlValue} onSave={handleSaveUrl} />
        </>
      )}
    </ThemeProvider>
  );
}

export default App;
