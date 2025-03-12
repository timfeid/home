<script lang="ts">
	export let activeServer: number;
	export let setActiveServer: (index: number) => void;

	// Import Svelte versions of the lucide icons
	import { Home, Plus, Compass, Download } from 'lucide-svelte';
	import { Tooltip, TooltipProvider, TooltipTrigger } from '../../ui/tooltip';
	import { cn } from '../../../utils';
	import TooltipContent from '../../ui/tooltip/tooltip-content.svelte';

	// Define the list of servers. For servers with an icon, store the component reference.
	const servers = [
		{ name: 'Home', icon: Home },
		{ name: 'Gaming', initial: 'G', color: 'bg-indigo-500' },
		{ name: 'Coding', initial: 'C', color: 'bg-green-500' },
		{ name: 'Music', initial: 'M', color: 'bg-pink-500' },
		{ name: 'Art', initial: 'A', color: 'bg-yellow-500' },
	];
</script>

<div class="flex w-[72px] flex-col items-center space-y-2 overflow-y-auto bg-[#202225] py-3">
	{#each servers as server, index (index)}
		<TooltipProvider delayDuration={300}>
			<Tooltip>
				<TooltipTrigger asChild>
					<button
						on:click={() => setActiveServer(index)}
						class={cn(
							'group relative mb-2 flex h-12 w-12 items-center justify-center rounded-full transition-all duration-200 hover:rounded-2xl',
							index === activeServer
								? server.color || 'bg-[#5865f2]'
								: 'bg-[#36393f] hover:bg-[#5865f2]',
						)}
					>
						<div
							class="absolute -left-3 h-0 w-1 rounded-r-full bg-white transition-all duration-200 group-hover:h-5"
						>
							{#if index === activeServer}
								<div class="h-10 w-1 rounded-r-full bg-white"></div>
							{/if}
						</div>
						{#if server.icon}
							<svelte:component this={server.icon} class="h-5 w-5" />
						{:else}
							<span class="text-xl font-semibold text-white">{server.initial}</span>
						{/if}
					</button>
				</TooltipTrigger>
				<TooltipContent side="right">
					<p>{server.name}</p>
				</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	{/each}

	<div class="my-1 h-[2px] w-8 rounded-full bg-[#36393f]"></div>

	<TooltipProvider delayDuration={300}>
		<Tooltip>
			<TooltipTrigger asChild>
				<button
					class="flex h-12 w-12 items-center justify-center rounded-full bg-[#36393f] text-green-500 transition-all duration-200 hover:rounded-2xl hover:bg-green-500 hover:text-white"
				>
					<Plus class="h-5 w-5" />
				</button>
			</TooltipTrigger>
			<TooltipContent side="right">
				<p>Add a Server</p>
			</TooltipContent>
		</Tooltip>
	</TooltipProvider>

	<TooltipProvider delayDuration={300}>
		<Tooltip>
			<TooltipTrigger>
				<button
					class="flex h-12 w-12 items-center justify-center rounded-full bg-[#36393f] text-[#5865f2] transition-all duration-200 hover:rounded-2xl hover:bg-[#5865f2] hover:text-white"
				>
					<Compass class="h-5 w-5" />
				</button>
			</TooltipTrigger>
			<TooltipContent side="right">
				<p>Explore Servers</p>
			</TooltipContent>
		</Tooltip>
	</TooltipProvider>

	<TooltipProvider delayDuration={300}>
		<Tooltip>
			<TooltipTrigger asChild>
				<button
					class="flex h-12 w-12 items-center justify-center rounded-full bg-[#36393f] text-[#5865f2] transition-all duration-200 hover:rounded-2xl hover:bg-[#5865f2] hover:text-white"
				>
					<Download class="h-5 w-5" />
				</button>
			</TooltipTrigger>
			<TooltipContent side="right">
				<p>Download Apps</p>
			</TooltipContent>
		</Tooltip>
	</TooltipProvider>
</div>
