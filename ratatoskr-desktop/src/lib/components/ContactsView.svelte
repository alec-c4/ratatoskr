<script lang="ts">
  import { Users, UserPlus, MessageSquare, Pencil, Trash2, X, Check } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  export let contacts: [string, string | null][];
  export let showAddContact: boolean;
  export let newContactDid: string;
  export let newContactAlias: string;

  export let onAddContact: () => void;
  export let onUpdateContact: (did: string, alias: string) => void;
  export let onDeleteContact: (did: string) => void;
  export let onOpenChat: (contact: [string, string | null]) => void;
  export let onToggleAdd: () => void;

  let editingContactDid: string | null = null;
  let editAlias = "";

  function startEdit(did: string, alias: string | null) {
    editingContactDid = did;
    editAlias = alias || "";
  }

  function cancelEdit() {
    editingContactDid = null;
    editAlias = "";
  }

  function saveEdit(did: string) {
    onUpdateContact(did, editAlias);
    cancelEdit();
  }
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
            <div class="flex items-center bg-muted/20 p-3 rounded-lg gap-4 group">
                <div class="w-10 h-10 rounded-full bg-muted flex items-center justify-center font-bold text-muted-foreground">
                    {alias ? alias[0].toUpperCase() : '?'}
                </div>
                <div class="flex-1 min-w-0">
                    {#if editingContactDid === did}
                        <div class="flex gap-2">
                            <Input bind:value={editAlias} class="h-8 text-sm" />
                            <Button size="icon" class="h-8 w-8" onclick={() => saveEdit(did)}><Check size={14}/></Button>
                            <Button size="icon" variant="ghost" class="h-8 w-8" onclick={cancelEdit}><X size={14}/></Button>
                        </div>
                    {:else}
                        <div class="font-bold truncate">{alias || "Unknown"}</div>
                        <div class="text-[10px] text-muted-foreground font-mono truncate">{did}</div>
                    {/if}
                </div>
                {#if editingContactDid !== did}
                    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-foreground" title="Message" onclick={() => onOpenChat([did, alias])}>
                            <MessageSquare size={16}/>
                        </Button>
                        <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-foreground" title="Edit" onclick={() => startEdit(did, alias)}>
                            <Pencil size={16}/>
                        </Button>
                        <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-destructive" title="Delete" onclick={() => onDeleteContact(did)}>
                            <Trash2 size={16}/>
                        </Button>
                    </div>
                {/if}
            </div>
        {/each}
        {#if contacts.length === 0}
            <div class="text-center text-muted-foreground py-10">No contacts yet. Add one to start chatting.</div>
        {/if}
    </div>
</div>
