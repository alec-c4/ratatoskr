<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { 
    Activity, 
    ShieldAlert, 
    Ambulance, 
    Flame, 
    Users, 
    MapPin, 
    Radio, 
    Settings, 
    MessageSquare,
    Menu,
    Terminal,
    Fingerprint,
    UserPlus,
    Download,
    Trash2,
    LogOut,
    LogIn
  } from "lucide-svelte";
  import { onMount } from "svelte";

  // Interfaces
  interface ChatMessage {
    id: string;
    sender_did: string;
    content: number[];
    timestamp: number;
    msg_type?: string;
    status?: string;
    ttl?: number;
  }

  // State
  let coreStatus = $state("Initializing...");
  let peersCount = $state(0);
  let activeTab = $state("sos");
  let logs: string[] = $state([]);
  let userIdentity = $state<string | null>(null);
  let accountCreated = $state(false);
  
  // Registration State
  let showRecovery = $state(false);
  let recoveryPhrase = $state("");
  let generatedMnemonic = $state("");
  let copyFeedback = $state("");

  // Contact State
  let contacts: [string, string | null][] = $state([]);
  let showAddContact = $state(false);
  let newContactDid = $state("");
  let newContactAlias = $state("");

  // Chat State
  let selectedContact = $state<[string, string | null] | null>(null);
  let chatMessages = $state<ChatMessage[]>([]);
  let newMessage = $state("");
  let nextMessageType = $state("Direct");

  // Initialization
  onMount(() => {
    checkCore();
    loadIdentity();
    loadContacts();

    // Listen for incoming messages
    let unlisten: () => void;
    listen<ChatMessage>("msg-received", (event) => {
      const msg = event.payload;
      addLog(`Message received from ${msg.sender_did}`);

      // If chat is open, append
      if (selectedContact && selectedContact[0] === msg.sender_did) {
        chatMessages = [...chatMessages, msg];
      }
    }).then((u) => (unlisten = u));

    // Simulate finding peers for UI demo
    const interval = setInterval(() => {
      if (peersCount < 5) peersCount += 1;
    }, 2000);

    return () => {
      clearInterval(interval);
      if (unlisten) unlisten();
    };
  });

  async function loadMessages(did: string) {
    try {
      chatMessages = await invoke("get_messages", { did });
    } catch (e) {
      addLog(`Messages Error: ${e}`);
    }
  }

  async function sendMessage() {
    if (!selectedContact || !newMessage) return;
    try {
        await invoke("send_message", { 
            recipientDid: selectedContact[0], 
            content: newMessage,
            // To be added to backend send_message call
        });
        await loadMessages(selectedContact[0]);
        newMessage = "";
    } catch (e) {
        addLog(`Send Error: ${e}`);
    }
  }

  async function markAsDone(msgId: string) {
    try {
        await invoke("update_message_status", { id: msgId, status: "Done" });
        if (selectedContact) await loadMessages(selectedContact[0]);
    } catch (e) {
        addLog(`Status Update Error: ${e}`);
    }
  }

  async function selectChat(contact: [string, string | null]) { // eslint-disable-line
    selectedContact = contact;
    await loadMessages(contact[0]);
  }

  async function loadContacts() {
    try {
        contacts = await invoke("get_contacts");
    } catch (e) {
        addLog(`Contacts Error: ${e}`);
    }
  }

  async function addContact() {
    if (!newContactDid) return;
    try {
        await invoke("add_contact", { did: newContactDid, alias: newContactAlias || "Unknown" });
        await loadContacts();
        showAddContact = false;
        newContactDid = "";
        newContactAlias = "";
        addLog(`Contact added: ${newContactAlias}`);
    } catch (e) {
        addLog(`Add Contact Error: ${e}`);
    }
  }

  async function loadIdentity() {
    try {
      const id = await invoke<string | null>("get_identity");
      if (id) {
        userIdentity = id;
        accountCreated = true;
        addLog(`Identity: Loaded key ${id.substring(0, 8)}...`);
      } else {
        userIdentity = null;
        accountCreated = false;
        addLog("Identity: No identity found.");
      }
    } catch (e) {
      addLog(`Identity Error: ${e}`);
    }
  }

  async function logout() {
    userIdentity = null;
    activeTab = "sos"; // Will show overlay
    addLog("System: Logged out.");
  }

  async function registerIdentity() {
    try {
      const [pubKey, mnemonic] = await invoke<[string, string]>("create_identity");
      userIdentity = pubKey;
      accountCreated = true;
      generatedMnemonic = mnemonic;
      addLog(`Identity: Created new key ${userIdentity.substring(0, 8)}...`);
    } catch (e) {
      addLog(`Registration Error: ${e}`);
    }
  }

  async function recoverIdentity() {
    try {
      const pubKey = await invoke<string>("recover_identity", { phrase: recoveryPhrase });
      userIdentity = pubKey;
      accountCreated = true;
      showRecovery = false;
      addLog(`Identity: Recovered key ${userIdentity.substring(0, 8)}...`);
    } catch (e) {
      addLog(`Recovery Error: ${e}`);
    }
  }

  async function panicWipe() {
    console.log("Panic Wipe triggered");
    // Removed confirm for debugging purposes - click destroys immediately
    try {
        await invoke("delete_identity");
        console.log("Identity deleted via Rust");
        userIdentity = null;
        accountCreated = false;
        generatedMnemonic = "";
        window.location.reload();
    } catch (e) {
        console.error("Wipe Error:", e);
        addLog(`Wipe Error: ${e}`);
    }
  }

  async function exportPublicKey() {
    if (!userIdentity) return;
    const content = `Ratatoskr Public Key (DID):\n\n${userIdentity}`;
    try {
        const path = await invoke<string>("export_backup", { content });
        alert(`Saved to: ${path}`);
    } catch (e) {
        alert(`Error: ${e}`);
    }
  }

  async function downloadBackup() {
    const content = `Ratatoskr Recovery Phrase:\n\n${generatedMnemonic}\n\nKEEP THIS SECRET!`;
    try {
        const path = await invoke<string>("export_backup", { content });
        addLog(`System: Backup saved to ${path}`);
        copyFeedback = "Saved to Downloads!";
    } catch (e) {
        addLog(`Backup Error: ${e}`);
        copyFeedback = "Save Failed";
    }
  }

  function copyMnemonic() {
     navigator.clipboard.writeText(generatedMnemonic);
     copyFeedback = "Copied!";
     setTimeout(() => copyFeedback = "", 2000);
  }

  async function checkCore() {
    try {
      const res = await invoke("ping");
      coreStatus = "Online";
      addLog(`System: ${res}`);
    } catch (e) {
      coreStatus = "Offline";
      addLog(`System Error: ${e}`);
    }
  }

  function addLog(msg: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [`[${timestamp}] ${msg}`, ...logs];
  }

  async function handleSos(type: string, desc: string) {
    addLog(`Encrypting SOS packet (${type})...`);
    try {
      // Real Tauri Call
      const resp = await invoke("send_sos", { 
        helpType: type, 
        lat: 55.75, 
        long: 37.61, 
        description: desc 
      });
      addLog(`Network: ${resp}`);
    } catch (e) {
      addLog(`Error: ${e}`);
    }
  }
</script>

<div class="app-layout">
  <!-- SIDEBAR -->
  <aside class="sidebar">
    <div class="logo">
      <Radio color="#27ae60" size={32} />
    </div>
    
    <nav>
      <button class:active={activeTab === 'chats'} onclick={() => activeTab = 'chats'} title="Chats">
        <MessageSquare size={24} />
      </button>
      <button class:active={activeTab === 'contacts'} onclick={() => activeTab = 'contacts'} title="Contacts">
        <Users size={24} />
      </button>
      <button class:active={activeTab === 'sos'} onclick={() => activeTab = 'sos'} title="SOS / Emergency" class="sos-tab">
        <ShieldAlert size={24} />
      </button>
      <button class:active={activeTab === 'settings'} onclick={() => activeTab = 'settings'} title="Settings">
        <Settings size={24} />
      </button>
    </nav>

    <div class="version">v0.1</div>
  </aside>

  <!-- MAIN CONTENT -->
  <main class="main-content">
    
    <!-- TOP BAR -->
    <header class="top-bar">
      <div class="status-indicator">
        <div class="dot" class:online={coreStatus === 'Online'}></div>
        <span>Core: {coreStatus}</span>
      </div>
      <div class="peers-indicator">
        <Activity size={16} />
        <span>{peersCount} Peers Connected</span>
      </div>
    </header>

    <!-- CONTENT AREA -->
    <div class="content-scroll">
      {#if !userIdentity && activeTab !== 'settings'}
        <div class="welcome-overlay">
            <Radio size={64} color="#27ae60" />
            <h2>Welcome to Ratatoskr</h2>
            
            {#if accountCreated}
                <p>Identity found on this device.</p>
                <div class="auth-buttons">
                    <button class="primary-btn" onclick={loadIdentity}>
                        <LogIn size={18} />
                        <span>Log In</span>
                    </button>
                </div>
            {:else}
                <p>To start communicating securely, you need to create an identity.</p>
                <div class="auth-buttons">
                    <button class="primary-btn" onclick={() => activeTab = 'settings'}>Get Started</button>
                </div>
            {/if}
        </div>
      {:else if activeTab === 'sos'}
        <div class="sos-container">
          <div class="warning-header">
            <ShieldAlert size={48} color="#e74c3c" />
            <h1>EMERGENCY BROADCAST</h1>
            <p>Black Box Protocol active. Your location and identity are encrypted.</p>
          </div>

          <div class="sos-grid">
            <button class="sos-card medical" onclick={() => handleSos('Medical', 'Medical Assistance Required')}>
              <Ambulance size={40} />
              <h3>MEDICAL</h3>
              <p>Injury, Bleeding, Sick</p>
            </button>

            <button class="sos-card evac" onclick={() => handleSos('Evacuation', 'Evacuation Required')}>
              <MapPin size={40} />
              <h3>EVACUATION</h3>
              <p>Trapped, Transport needed</p>
            </button>

            <button class="sos-card food" onclick={() => handleSos('FoodWater', 'Supplies Required')}>
              <Menu size={40} />
              <h3>SUPPLIES</h3>
              <p>Food, Water, Meds</p>
            </button>

            <button class="sos-card violence" onclick={() => handleSos('Violence', 'Under Attack')}>
              <Flame size={40} />
              <h3>DANGER</h3>
              <p>Violence, Shelling</p>
            </button>
          </div>

          <div class="terminal-log">
            <div class="terminal-header">
              <Terminal size={14} />
              <span>SECURE LOG</span>
            </div>
            <div class="logs">
              {#each logs as log, i (i)}
                <div class="log-line">{log}</div>
              {/each}
              {#if logs.length === 0}
                <div class="log-line opacity-50">System ready. Waiting for input...</div>
              {/if}
            </div>
          </div>
        </div>
      {:else if activeTab === 'chats'}
        <div class="chat-layout">
          <div class="chat-sidebar">
            <div class="section-header">Recent Chats</div>
            <div class="chat-list">
              {#each contacts as contact (contact[0])}
                <button
                  class="chat-item"
                  class:active={selectedContact?.[0] === contact[0]}
                  onclick={() => selectChat(contact)}
                >
                  <div class="avatar small">{contact[1]?.[0] || '?'}</div>
                  <div class="info">
                    <div class="name">{contact[1] || 'Unknown'}</div>
                    <div class="last-msg">No messages yet</div>
                  </div>
                </button>
              {/each}
              {#if contacts.length === 0}
                <p class="empty-state">No contacts found.</p>
              {/if}
            </div>
          </div>

          <div class="chat-main">
            {#if selectedContact}
              <header class="chat-header">
                <div class="avatar small">{selectedContact[1]?.[0] || '?'}</div>
                <span>{selectedContact[1]}</span>
                <span class="did-badge">{selectedContact[0].substring(0, 12)}...</span>
              </header>
              <div class="message-area">
                {#each chatMessages as msg (msg.id)}
                  <div
                    class="msg-bubble"
                    class:own={msg.sender_did === 'me'}
                    class:ephemeral={msg.msg_type === 'Ephemeral'}
                  >
                    <div class="msg-header">
                      <span class="type-tag">{msg.msg_type}</span>
                      {#if msg.status !== 'Done'}
                        <button class="done-btn" onclick={() => markAsDone(msg.id)} title="Mark as Done"
                          >✓</button
                        >
                      {/if}
                    </div>
                    <div class="msg-content">
                      {new TextDecoder().decode(new Uint8Array(msg.content))}
                    </div>
                    <div class="time">
                      {msg.status} • {new Date(msg.timestamp * 1000).toLocaleTimeString()}
                      {#if msg.ttl}
                        • Exp: {new Date(msg.ttl * 1000).toLocaleTimeString()}
                      {/if}
                    </div>
                  </div>
                {/each}
                {#if chatMessages.length === 0}
                  <div class="empty-state">Secure connection established. Start typing...</div>
                {/if}
              </div>
              <form
                class="input-area"
                onsubmit={(e) => {
                  e.preventDefault();
                  sendMessage();
                }}
              >
                <select bind:value={nextMessageType} class="type-select">
                  <option value="Direct">Direct</option>
                  <option value="Ephemeral">Ephemeral (1m)</option>
                  <option value="Transactional">Transactional</option>
                </select>
                <input placeholder="Type a secure message..." bind:value={newMessage} />
                <button type="submit">Send</button>
              </form>
            {:else}
              <div class="placeholder-view">
                <MessageSquare size={48} />
                <p>Select a contact to start chatting</p>
              </div>
            {/if}
          </div>
        </div>
      {:else if activeTab === 'contacts'}
        <div class="settings-view">
            <div class="section-header">
              <Users size={20} />
              <span>Contacts</span>
              <button class="small-btn" onclick={() => showAddContact = !showAddContact} style="margin-left: auto;">
                <UserPlus size={14} /> <span>Add</span>
              </button>
            </div>

            {#if showAddContact}
                <div class="add-contact-form">
                    <input placeholder="Public Key (DID)" bind:value={newContactDid} />
                    <input placeholder="Alias (Name)" bind:value={newContactAlias} />
                    <button class="primary-btn" onclick={addContact}>Save Contact</button>
                </div>
            {/if}

            <div class="contacts-list">
                {#each contacts as [did, alias] (did)}
                    <div class="contact-item">
                        <div class="avatar">
                            {alias ? alias[0].toUpperCase() : '?'}
                        </div>
                        <div class="info">
                            <div class="name">{alias || "Unknown"}</div>
                            <div class="did">{did.substring(0, 16)}...</div>
                        </div>
                        <div class="actions">
                            <button class="icon-btn" title="Message"><MessageSquare size={16}/></button>
                        </div>
                    </div>
                {/each}
                {#if contacts.length === 0}
                    <div class="empty-state">No contacts yet. Add one to start chatting.</div>
                {/if}
            </div>
        </div>
      {:else if activeTab === 'settings'}
        <div class="settings-view">
          <h2>SETTINGS</h2>
          
          <div class="settings-section">
            <div class="section-header">
              <Fingerprint size={20} />
              <span>Identity (DID)</span>
            </div>
            
            <div class="identity-card">
              {#if userIdentity}
                <div class="did-info">
                  <span class="label">Public Key (Hex)</span>
                  <div class="key-box">{userIdentity}</div>
                  <p class="hint">This is your unique decentralized ID on the network.</p>
                  
                  <button class="secondary-btn" onclick={exportPublicKey} style="width: 100%; margin-top: 10px;">
                    <Download size={14} /> <span>Export Public Key to File</span>
                  </button>

                  {#if generatedMnemonic}
                    <div class="mnemonic-alert">
                      <strong>⚠️ SECRET RECOVERY PHRASE</strong>
                      <p>Write this down immediately. It will not be shown again.</p>
                      <button class="mnemonic-box" onclick={copyMnemonic} type="button">
                        {generatedMnemonic}
                      </button>
                      <div class="mnemonic-actions">
                        <button class="download-btn" onclick={downloadBackup}>
                            <Download size={16} /> <span>Save Backup File</span>
                        </button>
                        <span class="copy-feedback">{copyFeedback}</span>
                      </div>
                    </div>
                  {/if}
                </div>
              {:else}
                {#if !showRecovery}
                  <div class="no-identity">
                    <p>You don't have an identity yet.</p>
                    <div class="auth-buttons">
                      <button class="primary-btn" onclick={registerIdentity}>
                        <UserPlus size={18} />
                        <span>Create New Account</span>
                      </button>
                      <button class="secondary-btn" onclick={() => showRecovery = true}>
                        <Fingerprint size={18} />
                        <span>I have a Recovery Phrase</span>
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="recovery-form">
                    <h3>Recover Account</h3>
                    <p>Enter your 12-word secret phrase below:</p>
                    <textarea 
                      bind:value={recoveryPhrase} 
                      placeholder="witch collapse practice feed shame open despair creek road again ice least"
                      rows="3"
                    ></textarea>
                    <div class="auth-buttons">
                      <button class="primary-btn" onclick={recoverIdentity}>Recover</button>
                      <button class="secondary-btn" onclick={() => showRecovery = false}>Cancel</button>
                    </div>
                  </div>
                {/if}
              {/if}
            </div>
          </div>

          <div class="settings-section">
             <div class="section-header">
              <Settings size={20} />
              <span>App Preferences</span>
            </div>
            <p class="opacity-50">More settings coming soon...</p>
            {#if userIdentity}
                <button class="secondary-btn" style="width: 100%; margin-top: 10px;" onclick={logout}>
                    <LogOut size={16} /> <span>Log Out</span>
                </button>
            {/if}
          </div>

          <div class="settings-section danger-zone">
             <div class="section-header danger">
              <ShieldAlert size={20} />
              <span>Danger Zone</span>
            </div>
            <p>Destructive actions. Use with extreme caution.</p>
            <button class="panic-btn" onclick={panicWipe}>
                <Trash2 size={18} />
                <span>Panic Wipe: Destroy Identity</span>
            </button>
          </div>
        </div>
      {:else}
        <div class="placeholder-view">
          <h2>{activeTab.toUpperCase()}</h2>
          <p>Module under construction.</p>
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
    background-color: #121212;
    color: #e0e0e0;
    overflow: hidden; /* App-like feel */
  }

  /* LAYOUT */
  .app-layout {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  /* SIDEBAR */
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

  .version {
    margin-top: auto;
    font-size: 10px;
    color: #444;
  }

  /* MAIN AREA */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #121212;
  }

  .top-bar {
    height: 50px;
    border-bottom: 1px solid #222;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    font-size: 12px;
    color: #888;
  }

  .status-indicator, .peers-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: #e74c3c;
  }

  .dot.online {
    background-color: #27ae60;
    box-shadow: 0 0 8px #27ae60aa;
  }

  .content-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  /* SOS VIEW */
  .sos-container {
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  .warning-header {
    text-align: center;
    margin-top: 20px;
  }

  .warning-header h1 {
    color: #e74c3c;
    margin: 10px 0 5px;
    letter-spacing: 2px;
  }

  .warning-header p {
    color: #888;
    margin: 0;
  }

  .sos-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 15px;
  }

  .sos-card {
    background-color: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 30px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    color: #eee;
  }

  .sos-card h3 { margin: 15px 0 5px; font-size: 18px; }
  .sos-card p { margin: 0; font-size: 12px; color: #888; }

  .sos-card:hover {
    transform: translateY(-2px);
    border-color: #555;
  }

  .sos-card:active {
    transform: translateY(1px);
    opacity: 0.8;
  }

  .sos-card.medical:hover { border-color: #e74c3c; color: #e74c3c; }
  .sos-card.evac:hover { border-color: #f39c12; color: #f39c12; }
  .sos-card.food:hover { border-color: #3498db; color: #3498db; }
  .sos-card.violence:hover { border-color: #9b59b6; color: #9b59b6; }

  /* TERMINAL */
  .terminal-log {
    background-color: #000;
    border: 1px solid #333;
    border-radius: 6px;
    margin-top: 20px;
    font-family: 'Courier New', Courier, monospace;
    font-size: 12px;
    display: flex;
    flex-direction: column;
  }

  .terminal-header {
    background-color: #222;
    padding: 5px 10px;
    color: #aaa;
    font-size: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    border-bottom: 1px solid #333;
  }

  .logs {
    padding: 10px;
    height: 150px;
    overflow-y: auto;
    color: #27ae60;
  }

  .log-line {
    margin-bottom: 4px;
    border-bottom: 1px solid #111;
  }

  .placeholder-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #444;
  }

  .welcome-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    animation: fadeIn 0.5s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .add-contact-form {
    background: #111;
    padding: 15px;
    border-radius: 8px;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .add-contact-form input {
    background: #000;
    border: 1px solid #333;
    color: white;
    padding: 10px;
    border-radius: 4px;
  }

  .contacts-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .contact-item {
    display: flex;
    align-items: center;
    background: #1a1a1a;
    padding: 10px;
    border-radius: 8px;
    gap: 15px;
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
    color: #888;
  }

  .info {
    flex: 1;
  }

  .name {
    font-weight: bold;
    color: #eee;
  }

  .did {
    font-size: 10px;
    color: #666;
    font-family: monospace;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: #666;
    cursor: pointer;
  }
  
  .icon-btn:hover {
    color: #27ae60;
  }

  .empty-state {
    text-align: center;
    color: #555;
    padding: 40px;
  }

  /* CHAT LAYOUT */
  .chat-layout {
    display: flex;
    height: 100%;
    margin: -20px; /* Counter container padding */
  }

  .chat-sidebar {
    width: 260px;
    background: #111;
    border-right: 1px solid #222;
    display: flex;
    flex-direction: column;
  }

  .chat-list {
    flex: 1;
    overflow-y: auto;
  }

  .chat-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 15px;
    background: transparent;
    border: none;
    border-bottom: 1px solid #222;
    text-align: left;
    cursor: pointer;
    transition: background 0.2s;
  }

  .chat-item:hover { background: #1a1a1a; }
  .chat-item.active { background: #1a1a1a; border-left: 3px solid #27ae60; }

  .chat-item .name { font-size: 14px; font-weight: bold; color: #fff; }
  .chat-item .last-msg { font-size: 11px; color: #666; }

  .chat-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #121212;
  }

  .chat-header {
    height: 60px;
    border-bottom: 1px solid #222;
    display: flex;
    align-items: center;
    padding: 0 20px;
    gap: 15px;
    background: #0a0a0a;
  }

  .did-badge {
    font-size: 10px;
    background: #222;
    padding: 2px 6px;
    border-radius: 4px;
    color: #666;
    font-family: monospace;
  }

  .message-area {
    flex: 1;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
  }

  .msg-bubble {
    max-width: 70%;
    padding: 10px 15px;
    background: #222;
    border-radius: 12px 12px 12px 2px;
    align-self: flex-start;
    font-size: 14px;
    position: relative;
    border: 1px solid #333;
  }

  .msg-bubble.ephemeral {
    border-color: #f1c40f55;
    background: #2c2c00;
  }

  .msg-header {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    margin-bottom: 5px;
    opacity: 0.7;
  }

  .type-tag {
    background: #333;
    padding: 1px 4px;
    border-radius: 3px;
    text-transform: uppercase;
  }

  .done-btn {
    background: transparent;
    border: none;
    color: #27ae60;
    cursor: pointer;
    font-weight: bold;
    padding: 0 4px;
  }

  .done-btn:hover {
    transform: scale(1.2);
  }

  .type-select {
    background: #111;
    color: #888;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 11px;
    padding: 0 5px;
  }

  .msg-bubble.own {
    background: #27ae60;
    color: #fff;
    align-self: flex-end;
    border-radius: 12px 12px 2px 12px;
  }

  .time {
    font-size: 9px;
    opacity: 0.6;
    margin-top: 5px;
    text-align: right;
  }

  .input-area {
    padding: 20px;
    background: #0a0a0a;
    display: flex;
    gap: 10px;
    border-top: 1px solid #222;
  }

  .input-area input {
    flex: 1;
    background: #1a1a1a;
    border: 1px solid #333;
    color: #fff;
    padding: 10px 15px;
    border-radius: 20px;
  }

  .avatar.small { width: 32px; height: 32px; font-size: 12px; }

  /* SETTINGS & IDENTITY */
  .settings-view {
    max-width: 600px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  .settings-section {
    background-color: #1a1a1a;
    border: 1px solid #333;
    border-radius: 12px;
    padding: 20px;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
    color: #27ae60;
    font-weight: bold;
    text-transform: uppercase;
    font-size: 14px;
    border-bottom: 1px solid #333;
    padding-bottom: 10px;
  }

  .identity-card {
    background-color: #111;
    border-radius: 8px;
    padding: 15px;
  }

  .key-box {
    background-color: #000;
    padding: 12px;
    border-radius: 6px;
    font-family: monospace;
    font-size: 12px;
    color: #27ae60;
    word-break: break-all;
    border: 1px solid #222;
    margin: 10px 0;
  }

  .label {
    font-size: 11px;
    color: #666;
    text-transform: uppercase;
  }

  .hint {
    font-size: 12px;
    color: #555;
    margin: 0;
  }

  .no-identity {
    text-align: center;
    padding: 20px 0;
  }

  .primary-btn {
    background-color: #27ae60;
    color: white;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px 24px;
    font-weight: bold;
    margin: 20px auto 0;
  }

  .primary-btn:hover {
    background-color: #2ecc71;
    border-color: transparent;
  }

  .auth-buttons {
    display: flex;
    gap: 10px;
    justify-content: center;
    margin-top: 20px;
  }

  .secondary-btn {
    background-color: transparent;
    border: 1px solid #444;
    color: #aaa;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px 24px;
    cursor: pointer;
  }
  
  .secondary-btn:hover {
    border-color: #888;
    color: #fff;
  }

  .mnemonic-alert {
    background-color: #2c1e00;
    border: 1px solid #f39c12;
    padding: 15px;
    border-radius: 8px;
    margin-top: 20px;
    color: #f39c12;
  }

  .mnemonic-box {
    background-color: #000;
    padding: 15px;
    font-family: monospace;
    font-size: 14px;
    color: #fff;
    border: 1px dashed #f39c12;
    margin-top: 10px;
    cursor: pointer;
    line-height: 1.5;
    width: 100%;
    text-align: left;
    display: block;
  }
  
  .mnemonic-box:hover {
    background-color: #111;
  }
  
  .copy-feedback {
    font-size: 10px;
    color: #27ae60;
  }

  .mnemonic-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 10px;
  }

  .download-btn {
    background: #27ae60;
    color: #fff;
    font-size: 12px;
    padding: 8px 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }

  .download-btn:hover {
    background: #2ecc71;
  }

  .danger-zone {
    border-color: #441111;
    background-color: #1a0a0a;
  }

  .section-header.danger {
    color: #e74c3c;
    border-color: #441111;
  }

  .panic-btn {
    background-color: #e74c3c;
    color: white;
    width: 100%;
    margin-top: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px;
    font-weight: bold;
    border: none;
  }

  .panic-btn:hover {
    background-color: #c0392b;
  }

  textarea {
    width: 100%;
    background: #000;
    border: 1px solid #333;
    color: white;
    padding: 10px;
    border-radius: 6px;
    resize: none;
    font-family: monospace;
    margin-top: 10px;
  }
</style>
