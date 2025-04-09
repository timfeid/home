<script lang="ts">
  import { Avatar, AvatarImage, AvatarFallback } from '$lib/components/ui/avatar';
  import { Input } from '$lib/components/ui/input';
  import { Hash, Search, Bell, Pin, Users } from 'lucide-svelte';

  let searchVisible = $state(false);

  function setSearchVisible(visible: boolean) {
    searchVisible = visible;
  }
</script>

<header class="flex h-12 flex-shrink-0 items-center justify-between border-b bg-background px-4">
  <div class="flex items-center">
    <Avatar class="mr-2 h-6 w-6">
      <AvatarImage src="/placeholder.svg?height=24&width=24" alt="Devils" />
      <AvatarFallback class="bg-teal-900 text-xs text-zinc-300">NJ</AvatarFallback>
    </Avatar>
    <Hash class="mr-2 h-4 w-4 text-zinc-400" />
    <span class="font-mono text-teal-400">Devils News</span>
    <span class="mx-2 text-zinc-500">|</span>
    <span class="font-mono text-xs text-zinc-400">Hockey • NHL</span>
  </div>

  <div class="flex items-center space-x-3">
    {#if searchVisible}
      <div class="relative">
        <Input
          class="h-7 w-48  bg-teal-900 font-mono text-xs"
          placeholder="Search in channel..."
          onblur={() => setSearchVisible(false)}
          autofocus
        />
        <Search
          class="absolute right-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 transform text-zinc-400"
        />
      </div>
    {:else}
      <button class="text-zinc-400 hover:text-teal-400" onclick={() => setSearchVisible(true)}>
        <Search class="h-4 w-4" />
      </button>
      <button class="text-zinc-400 hover:text-teal-400">
        <Bell class="h-4 w-4" />
      </button>
      <button class="text-zinc-400 hover:text-teal-400">
        <Pin class="h-4 w-4" />
      </button>
      <button class="text-zinc-400 hover:text-teal-400">
        <Users class="h-4 w-4" />
      </button>
    {/if}
  </div>
</header>
