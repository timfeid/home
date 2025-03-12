<script lang="ts">
	export let activeChannel: number;
	export let setActiveChannel: (index: number) => void;
	export let onClose: () => void;

	// Import Svelte icon components (assumes lucide-svelte)
	import { ChevronDown, Hash, Volume2, Settings, X, Users, Plus } from 'lucide-svelte';
	import ScrollArea from '../../ui/scroll-area/scroll-area.svelte';
	import { Avatar, AvatarFallback, AvatarImage } from '../../ui/avatar';
	import { cn } from '../../../utils';

	// Local state for expanded/collapsed categories
	let expandedCategories = {
		text: true,
		voice: true,
	};

	function toggleCategory(category: string) {
		expandedCategories = { ...expandedCategories, [category]: !expandedCategories[category] };
	}

	// Data arrays for channels
	const textChannels = [
		{ name: 'general', unread: true, mentions: 0 },
		{ name: 'welcome', unread: false, mentions: 0 },
		{ name: 'announcements', unread: true, mentions: 3 },
		{ name: 'off-topic', unread: false, mentions: 0 },
	];

	const voiceChannels = [
		{ name: 'General', users: 2 },
		{ name: 'Gaming', users: 0 },
		{ name: 'Music', users: 1 },
	];
</script>

<div class="flex w-60 flex-col bg-[#2f3136]">
	<!-- Header -->
	<div class="flex h-12 items-center justify-between border-b border-[#202225] px-4 shadow-sm">
		<h2 class="truncate font-semibold text-white">Coding Server</h2>
		<button on:click={onClose} class="text-gray-400 hover:text-white md:hidden">
			<X class="h-5 w-5" />
		</button>
	</div>

	<!-- Main Scroll Area -->
	<ScrollArea class="flex-1">
		<div class="p-2">
			<!-- Text Channels -->
			<div class="mb-2">
				<button
					on:click={() => toggleCategory('text')}
					class="flex w-full items-center px-1 py-1.5 text-xs font-semibold text-gray-400 hover:text-gray-300"
				>
					<ChevronDown
						class={cn(
							'mr-0.5 h-3 w-3 transition-transform',
							!expandedCategories.text && '-rotate-90 transform',
						)}
					/>
					TEXT CHANNELS
					<Plus class="ml-auto h-3.5 w-3.5 text-gray-400 hover:text-gray-300" />
				</button>

				{#if expandedCategories.text}
					<div class="mt-1 space-y-0.5">
						{#each textChannels as channel, index}
							<button
								on:click={() => setActiveChannel(index)}
								class={cn(
									'group flex w-full items-center rounded px-2 py-1',
									activeChannel === index
										? 'bg-[#393c43] text-white'
										: 'text-gray-400 hover:bg-[#393c43] hover:text-gray-300',
								)}
							>
								<Hash class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400" />
								<span
									class={cn(
										'truncate',
										channel.unread && activeChannel !== index && 'font-semibold text-white',
									)}
								>
									{channel.name}
								</span>
								{#if channel.mentions > 0}
									<span class="ml-auto rounded-full bg-red-500 px-1.5 py-0.5 text-xs text-white">
										{channel.mentions}
									</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Voice Channels -->
			<div class="mb-2">
				<button
					on:click={() => toggleCategory('voice')}
					class="flex w-full items-center px-1 py-1.5 text-xs font-semibold text-gray-400 hover:text-gray-300"
				>
					<ChevronDown
						class={cn(
							'mr-0.5 h-3 w-3 transition-transform',
							!expandedCategories.voice && '-rotate-90 transform',
						)}
					/>
					VOICE CHANNELS
					<Plus class="ml-auto h-3.5 w-3.5 text-gray-400 hover:text-gray-300" />
				</button>

				{#if expandedCategories.voice}
					<div class="mt-1 space-y-0.5">
						{#each voiceChannels as channel, index}
							<div class="px-2">
								<button
									class="group flex w-full items-center rounded px-2 py-1 text-gray-400 hover:bg-[#393c43] hover:text-gray-300"
								>
									<Volume2 class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400" />
									<span class="truncate">{channel.name}</span>
									{#if channel.users > 0}
										<div class="ml-auto flex items-center">
											<Users class="mr-1 h-3.5 w-3.5 text-gray-400" />
											<span class="text-xs">{channel.users}</span>
										</div>
									{/if}
								</button>

								{#if channel.users > 0}
									<div class="ml-9 mt-1 space-y-1">
										{#each Array(channel.users) as _, i}
											<div class="flex items-center text-sm text-gray-400">
												<div class="relative mr-2">
													<Avatar class="h-6 w-6">
														<AvatarImage src={`/placeholder.svg?height=24&width=24`} />
														<AvatarFallback class="text-xs"
															>{String.fromCharCode(65 + i)}</AvatarFallback
														>
													</Avatar>
													<div
														class="absolute bottom-0 right-0 h-2 w-2 rounded-full bg-green-500 ring-1 ring-[#2f3136]"
													></div>
												</div>
												<span class="text-xs">User {i + 1}</span>
											</div>
										{/each}
									</div>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</ScrollArea>

	<!-- Footer/User Info -->
	<div class="flex h-14 items-center bg-[#292b2f] px-2">
		<div class="flex min-w-0 flex-1 items-center">
			<Avatar class="mr-2 h-8 w-8">
				<AvatarImage src="/placeholder.svg?height=32&width=32" />
				<AvatarFallback>U</AvatarFallback>
			</Avatar>
			<div class="min-w-0">
				<p class="truncate text-sm font-medium text-white">Username</p>
				<p class="truncate text-xs text-gray-400">#1234</p>
			</div>
		</div>
		<div class="flex space-x-1">
			<button class="text-gray-400 hover:text-gray-200">
				<Settings class="h-5 w-5" />
			</button>
		</div>
	</div>
</div>
