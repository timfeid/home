<script lang="ts">
  import { X } from 'lucide-svelte';

  let { onClose } = $props();

  let serverName = $state('irc.gaming.net');
  let nickname = $state('User123');
  let alternateNick = $state('User123_');
  let autoJoinChannels = $state('#gaming,#help');
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
  <div class="w-full max-w-md border-2 bg-gray-200">
    <div class="flex items-center justify-between bg-blue-800 p-1 text-white">
      <h2 class="text-sm font-bold">Server Settings</h2>
      <button onclick={onClose} class="text-white">
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="space-y-4 p-4">
      <div class="space-y-1">
        <label class="block text-sm font-bold">Server Address:</label>
        <input type="text" bind:value={serverName} class="w-full border p-1 text-sm" />
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Port:</label>
        <input type="text" value="6667" class="w-full border p-1 text-sm" />
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Nickname:</label>
        <input type="text" bind:value={nickname} class="w-full border p-1 text-sm" />
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Alternative Nickname:</label>
        <input type="text" bind:value={alternateNick} class="w-full border p-1 text-sm" />
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Auto-join channels:</label>
        <input type="text" bind:value={autoJoinChannels} class="w-full border p-1 text-sm" />
        <p class="text-xs text-gray-600">Separate multiple channels with commas</p>
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Connect on startup:</label>
        <div class="flex items-center">
          <input type="checkbox" class="mr-2" checked />
          <span class="text-sm">Automatically connect to this server on startup</span>
        </div>
      </div>

      <div class="space-y-1">
        <label class="block text-sm font-bold">Voice settings:</label>
        <div class="flex items-center">
          <input type="checkbox" class="mr-2" checked />
          <span class="text-sm">Auto-join voice when joining voice channels</span>
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-2 border-t bg-gray-300 p-2">
      <button onclick={onClose} class="border bg-gray-100 px-3 py-1 text-sm"> Cancel </button>
      <button class="bg-blue-600 px-3 py-1 text-sm text-white">Save</button>
    </div>
  </div>
</div>
