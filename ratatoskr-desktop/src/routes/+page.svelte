<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Activity } from "lucide-svelte";

  // Components
  import Sidebar from "$lib/components/Sidebar.svelte";
  import WelcomeView from "$lib/components/WelcomeView.svelte";
  import SOSView from "$lib/components/SOSView.svelte";
  import ContactsView from "$lib/components/ContactsView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import ChatView from "$lib/components/ChatView.svelte";

  // Interfaces
  interface ChatMessage {
    id: string;
    sender_did: string;
    content: number[];
    timestamp: number;
    msg_type?: string;
    status?: string;
    ttl?: number;
    reply_to_id?: string;
  }

  // State
  let coreStatus = $state("Initializing...");
  let peersCount = $state(0);
  let activeTab = $state("sos");
  let logs: string[] = $state([]);
  let userIdentity = $state<string | null>(null);
  let accountCreated = $state(false);

  // Settings State
  let showRecovery = $state(false);
  let recoveryPhrase = $state("");
  let generatedMnemonic = $state("");
  let copyFeedback = $state("");
  let registrationName = $state("");
  let userProfileName = $state("Anonymous");

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
  let replyToMessage = $state<ChatMessage | null>(null);

  // Initialization
  onMount(() => {
    checkCore();
    loadIdentity();
    loadContacts();

    let unlisten: () => void;
    listen<ChatMessage>("msg-received", (event) => {
      const msg = event.payload;
      addLog(`Message received from ${msg.sender_did}`);
      if (selectedContact && selectedContact[0] === msg.sender_did) {
        chatMessages = [...chatMessages, msg];
      }
    }).then((u) => (unlisten = u));

    listen<number>("peer-count-update", (event) => {
      peersCount = event.payload;
    });

    return () => {
      if (unlisten) unlisten();
    };
  });

  // UI Garbage Collector
  $effect(() => {
    const timer = setInterval(() => {
      const now = Date.now() / 1000;
      if (chatMessages.length > 0) {
        chatMessages = chatMessages.filter((msg) => !msg.ttl || msg.ttl > now);
      }
    }, 1000);
    return () => clearInterval(timer);
  });

  // --- ACTIONS ---

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
        msgTypeStr: nextMessageType,
        replyToId: replyToMessage ? replyToMessage.id : null,
      });
      await loadMessages(selectedContact[0]);
      newMessage = "";
      replyToMessage = null;
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

  async function selectChat(contact: [string, string | null]) {
    selectedContact = contact;
    await loadMessages(contact[0]);
  }

  async function openChat(contact: [string, string | null]) {
    activeTab = "chats";
    await selectChat(contact);
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
      await invoke("add_contact", {
        did: newContactDid,
        alias: newContactAlias || "Unknown",
      });
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
        userProfileName = await invoke("get_profile_name");
        addLog(`Identity: Loaded key ${id.substring(0, 8)}... as ${userProfileName}`);
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
    activeTab = "sos";
    addLog("System: Logged out.");
  }

  async function registerIdentity() {
    if (!registrationName) {
      alert("Please enter a name");
      return;
    }
    try {
      const [pubKey, mnemonic] = await invoke<[string, string]>("create_identity", {
        nickname: registrationName,
      });
      userIdentity = pubKey;
      userProfileName = registrationName;
      accountCreated = true;
      generatedMnemonic = mnemonic;
      addLog(`Identity: Created new key for ${registrationName}`);
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
    try {
      await invoke("delete_identity");
      window.location.reload();
    } catch (e) {
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
    setTimeout(() => (copyFeedback = ""), 2000);
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
    addLog(`Encrypting SOS packet (${type})...
`);
    try {
      const resp = await invoke("send_sos", {
        helpType: type,
        lat: 55.75,
        long: 37.61,
        description: desc,
      });
      addLog(`Network: ${resp}`);
    } catch (e) {
      addLog(`Error: ${e}`);
    }
  }
</script>

<div class="flex h-screen w-screen bg-background text-foreground overflow-hidden">
  <Sidebar 
    activeTab={activeTab} 
    userProfileName={userProfileName}
    onTabChange={(t) => activeTab = t}
  />

  <main class="flex-1 flex flex-col bg-background">
    <div class="h-[50px] border-b flex items-center justify-between px-5 text-xs text-muted-foreground">
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full" class:bg-green-500={coreStatus === 'Online'} class:bg-red-500={coreStatus !== 'Online'}></div>
        <span>Core: {coreStatus}</span>
      </div>
      <div class="flex items-center gap-2">
        <Activity size={16} />
        <span>{peersCount} Peers Connected</span>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-5">
      {#if !userIdentity && activeTab !== "settings"}
        <WelcomeView 
            accountCreated={accountCreated}
            onLoadIdentity={loadIdentity}
            onGetStarted={() => activeTab = 'settings'}
        />
      {:else if activeTab === "sos"}
        <SOSView 
            logs={logs}
            onHandleSos={handleSos}
        />
      {:else if activeTab === "chats"}
        <ChatView
            contacts={contacts}
            selectedContact={selectedContact}
            chatMessages={chatMessages}
            userIdentity={userIdentity}
            bind:newMessage={newMessage}
            bind:nextMessageType={nextMessageType}
            bind:replyToMessage={replyToMessage}
            onSelectChat={selectChat}
            onSendMessage={sendMessage}
            onMarkAsDone={markAsDone}
            onSetReply={(msg) => replyToMessage = msg}
        />
      {:else if activeTab === "contacts"}
        <ContactsView
            contacts={contacts}
            bind:showAddContact={showAddContact}
            bind:newContactDid={newContactDid}
            bind:newContactAlias={newContactAlias}
            onAddContact={addContact}
            onOpenChat={openChat}
            onToggleAdd={() => showAddContact = !showAddContact}
        />
      {:else if activeTab === "settings"}
        <SettingsView
            userIdentity={userIdentity}
            generatedMnemonic={generatedMnemonic}
            copyFeedback={copyFeedback}
            showRecovery={showRecovery}
            bind:registrationName={registrationName}
            bind:recoveryPhrase={recoveryPhrase}
            onRegister={registerIdentity}
            onRecover={recoverIdentity}
            onLogout={logout}
            onPanicWipe={panicWipe}
            onExportKey={exportPublicKey}
            onDownloadBackup={downloadBackup}
            onCopyMnemonic={copyMnemonic}
        />
      {/if}
    </div>
  </main>
</div>
