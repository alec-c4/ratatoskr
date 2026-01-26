<script lang="ts">
  import { Users, UserPlus, MessageSquare } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  export let contacts: [string, string | null][];
  export let showAddContact: boolean;
  export let newContactDid: string;
  export let newContactAlias: string;

  export let onAddContact: () => void;
  export let onOpenChat: (contact: [string, string | null]) => void;
  export let onToggleAdd: () => void;
</script>

<div class="max-w-[600px] mx-auto flex flex-col gap-8">
    <div class="flex items-center gap-2 border-b pb-2 text-primary font-bold uppercase text-sm">
      <Users size={20} />
      <span>Contacts</span>
      <Button variant="ghost" size="sm" onclick={onToggleAdd} class="ml-auto gap-2">
        <UserPlus size={14} /> Add
      </Button>
    </div>

    {#if showAddContact}
        <div class="bg-muted/30 p-4 rounded-lg mb-6 flex flex-col gap-3">
            <Input placeholder="Public Key (DID)" bind:value={newContactDid} />
            <Input placeholder="Alias (Name)" bind:value={newContactAlias} />
            <Button onclick={onAddContact}>Save Contact</Button>
        </div>
    {/if}

    <div class="flex flex-col gap-3">
        {#each contacts as [did, alias] (did)}
            <div class="flex items-center bg-muted/20 p-3 rounded-lg gap-4">
                <div class="w-10 h-10 rounded-full bg-muted flex items-center justify-center font-bold text-muted-foreground">
                    {alias ? alias[0].toUpperCase() : '?'}
                </div>
                <div class="flex-1">
                    <div class="font-bold">{alias || "Unknown"}</div>
                    <div class="text-[10px] text-muted-foreground font-mono">{did.substring(0, 16)}...</div>
                </div>
                <div class="actions">
                    <Button variant="ghost" size="icon" title="Message" onclick={() => onOpenChat([did, alias])}>
                        <MessageSquare size={16}/>
                    </Button>
                </div>
            </div>
        {/each}
        {#if contacts.length === 0}
            <div class="text-center text-muted-foreground py-10">No contacts yet. Add one to start chatting.</div>
        {/if}
    </div>
</div>
