<script lang="ts">
	export let showChannels: boolean;
	export let setShowChannels: (show: boolean) => void;
	export let showMembers: boolean;
	export let setShowMembers: (show: boolean) => void;

	// Using Svelte versions of lucide icons
	import { Menu, Hash, Users } from 'lucide-svelte';
	import { Button } from '../../ui/button';
</script>

<div class="flex h-12 items-center border-b border-[#202225] bg-[#36393f] px-4 shadow-sm">
	<Button
		variant="ghost"
		size="icon"
		class="mr-2 text-gray-200"
		on:click={() => setShowChannels(!showChannels)}
	>
		<Menu class="h-5 w-5" />
	</Button>

	<div class="flex items-center">
		<Hash class="mr-2 h-5 w-5 text-gray-400" />
		<h3 class="font-semibold text-white">general</h3>
	</div>

	<Button
		variant="ghost"
		size="icon"
		class="ml-auto text-gray-200"
		on:click={() => setShowMembers(!showMembers)}
	>
		<Users class="h-5 w-5" />
	</Button>
</div>
