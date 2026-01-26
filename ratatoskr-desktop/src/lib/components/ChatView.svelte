<script lang="ts">
  import { MessageSquare } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  // Type definitions
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

  export let contacts: [string, string | null][];
  export let selectedContact: [string, string | null] | null;
  export let chatMessages: ChatMessage[];
  export let newMessage: string;
  export let nextMessageType: string;
  export let replyToMessage: ChatMessage | null;
  export let userIdentity: string | null;

  export let onSelectChat: (contact: [string, string | null]) => void;
  export let onSendMessage: () => void;
  export let onMarkAsDone: (msgId: string) => void;
  export let onSetReply: (msg: ChatMessage | null) => void;
</script>

<div class="flex h-full -m-5">
  <!-- SIDEBAR -->
  <div class="w-[260px] bg-muted/10 border-r flex flex-col">
    <div class="p-4 border-b font-bold text-sm">Recent Chats</div>
    <div class="flex-1 overflow-y-auto">
      {#each contacts as contact (contact[0])}
        <button
          class="w-full flex items-center gap-3 p-4 border-b hover:bg-muted/20 transition-colors text-left"
          class:bg-muted={selectedContact?.[0] === contact[0]}
          class:border-l-4={selectedContact?.[0] === contact[0]}
          class:border-l-primary={selectedContact?.[0] === contact[0]}
          onclick={() => onSelectChat(contact)}
        >
          <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center text-xs font-bold">
            {contact[1]?.[0] || '?'}
          </div>
          <div class="overflow-hidden">
            <div class="font-bold text-sm truncate">{contact[1] || 'Unknown'}</div>
            <div class="text-[11px] text-muted-foreground">No messages yet</div>
          </div>
        </button>
      {/each}
      {#if contacts.length === 0}
        <p class="p-4 text-center text-muted-foreground text-sm">No contacts found.</p>
      {/if}
    </div>
  </div>

  <!-- MAIN CHAT -->
  <div class="flex-1 flex flex-col bg-background">
    {#if selectedContact}
      <header class="h-[60px] border-b flex items-center px-6 gap-4 bg-muted/5">
        <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center text-xs font-bold">
            {selectedContact[1]?.[0] || '?'}
        </div>
        <div>
            <div class="font-bold">{selectedContact[1]}</div>
            <span class="text-[10px] bg-muted px-1.5 py-0.5 rounded text-muted-foreground font-mono">
                {selectedContact[0].substring(0, 12)}...
            </span>
        </div>
      </header>
      
      <div class="flex-1 p-6 flex flex-col gap-3 overflow-y-auto">
        {#each chatMessages as msg (msg.id)}
          <div
            class="max-w-[70%] p-3 rounded-xl text-sm relative border"
            class:self-end={msg.sender_did === userIdentity || msg.sender_did === 'me'}
            class:bg-primary={msg.sender_did === userIdentity || msg.sender_did === 'me'}
            class:text-primary-foreground={msg.sender_did === userIdentity || msg.sender_did === 'me'}
            class:rounded-tr-sm={msg.sender_did === userIdentity || msg.sender_did === 'me'}
            class:self-start={msg.sender_did !== userIdentity && msg.sender_did !== 'me'}
            class:bg-muted={msg.sender_did !== userIdentity && msg.sender_did !== 'me'}
            class:rounded-tl-sm={msg.sender_did !== userIdentity && msg.sender_did !== 'me'}
            class:border-yellow-500={msg.msg_type === 'Ephemeral'}
            class:bg-yellow-950={msg.msg_type === 'Ephemeral'}
          >
            <div class="flex justify-between text-[9px] mb-1 opacity-70">
              <span class="bg-black/20 px-1 rounded uppercase">{msg.msg_type}</span>
              <div class="flex gap-1">
                <button class="hover:text-white font-bold" onclick={() => onSetReply(msg)} title="Reply">↩</button>
                {#if msg.status !== 'Done'}
                    <button class="hover:text-white font-bold" onclick={() => onMarkAsDone(msg.id)} title="Mark as Done">✓</button>
                {/if}
              </div>
            </div>
            
            {#if msg.reply_to_id}
                <div class="text-[10px] border-l-2 border-white/30 pl-2 mb-2 opacity-60">
                    Replying to message...
                </div>
            {/if}

            <div class="break-words">
              {new TextDecoder().decode(new Uint8Array(msg.content))}
            </div>
            <div class="text-[9px] opacity-60 text-right mt-1">
              {msg.status} • {new Date(msg.timestamp * 1000).toLocaleTimeString()}
              {#if msg.ttl}
                • Exp: {new Date(msg.ttl * 1000).toLocaleTimeString()}
              {/if}
            </div>
          </div>
        {/each}
        {#if chatMessages.length === 0}
          <div class="h-full flex items-center justify-center text-muted-foreground">Secure connection established. Start typing...</div>
        {/if}
      </div>

      <form
        class="p-4 bg-muted/10 border-t flex flex-col gap-2"
        onsubmit={(e) => {
          e.preventDefault();
          onSendMessage();
        }}
      >
        {#if replyToMessage}
            <div class="bg-muted/20 p-2 border-l-2 border-primary text-xs flex justify-between items-center text-muted-foreground">
                <span>Replying to message...</span>
                <button type="button" onclick={() => onSetReply(null)}>x</button>
            </div>
        {/if}
        <div class="flex gap-2">
            <select bind:value={nextMessageType} class="bg-background border border-input rounded h-10 px-2 text-xs w-24">
              <option value="Direct">Direct</option>
              <option value="Ephemeral">Ephemeral (1m)</option>
              <option value="Transactional">Transactional</option>
            </select>
            <Input placeholder="Type a secure message..." bind:value={newMessage} class="flex-1" />
            <Button type="submit">Send</Button>
        </div>
      </form>
    {:else}
      <div class="h-full flex flex-col items-center justify-center text-muted-foreground">
        <MessageSquare size={48} />
        <p class="mt-4">Select a contact to start chatting</p>
      </div>
    {/if}
  </div>
</div>
