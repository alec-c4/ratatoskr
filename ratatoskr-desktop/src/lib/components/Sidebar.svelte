<script lang="ts">
  import { 
    MessageSquare, 
    Users, 
    ShieldAlert, 
    Settings, 
    Radio 
  } from "lucide-svelte";

  export let activeTab: string;
  export let userProfileName: string;
  export let onTabChange: (tab: string) => void;
</script>

<aside class="sidebar">
  <div class="logo">
    <Radio color="#27ae60" size={32} />
  </div>
  
  <nav>
    <button class:active={activeTab === 'chats'} onclick={() => onTabChange('chats')} title="Chats">
      <MessageSquare size={24} />
    </button>
    <button class:active={activeTab === 'contacts'} onclick={() => onTabChange('contacts')} title="Contacts">
      <Users size={24} />
    </button>
    <button class:active={activeTab === 'sos'} onclick={() => onTabChange('sos')} title="SOS / Emergency" class="sos-tab">
      <ShieldAlert size={24} />
    </button>
    <button class:active={activeTab === 'settings'} onclick={() => onTabChange('settings')} title="Settings">
      <Settings size={24} />
    </button>
  </nav>

  <div class="profile-badge">
      <div class="avatar small">{userProfileName[0]}</div>
      <div class="p-name">{userProfileName}</div>
  </div>
</aside>

<style>
  .sidebar {
    width: 70px;
    background-color: #0a0a0a;
    border-right: 1px solid #333;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px 0;
  }

  .logo {
    margin-bottom: 40px;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 20px;
    width: 100%;
    align-items: center;
  }

  nav button {
    background: transparent;
    border: none;
    color: #666;
    padding: 12px;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  nav button:hover {
    color: #fff;
    background-color: #222;
  }

  nav button.active {
    color: #27ae60;
    background-color: #1a1a1a;
  }

  nav button.sos-tab.active {
    color: #e74c3c;
  }

  .profile-badge {
    margin-top: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    border-top: 1px solid #333;
    width: 100%;
  }

  .p-name {
    font-size: 10px;
    margin-top: 5px;
    color: #e5e5e5;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #333;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    color: #fff;
  }
  
  .avatar.small { width: 32px; height: 32px; font-size: 12px; }
</style>
