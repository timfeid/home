<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Hash,
		Bell,
		PinIcon,
		Users,
		Smile,
		PlusCircle,
		Gift,
		GiftIcon as GIF,
		Paperclip,
		Send,
	} from 'lucide-svelte';
	import { ScrollArea } from '../../ui/scroll-area';
	import { Avatar, AvatarFallback, AvatarImage } from '../../ui/avatar';
	import { Button } from '../../ui/button';
	import { Input } from '../../ui/input';

	// Import Svelte UI components (adjust paths to your project)

	export let activeServer: number;
	export let activeChannel: number;

	let message: string = '';
	let messages: any[] = [];
	let scrollAreaRef: HTMLDivElement;

	// Generate some mock messages on mount or when activeServer/activeChannel change.
	// This mimics the useEffect in React.
	$: {
		// You might want to prevent overriding messages sent by the user.
		// In this conversion, we re-generate messages whenever activeServer or activeChannel change.
		const serverNames = ['Home', 'Gaming', 'Coding', 'Music', 'Art'];
		const channelNames = ['general', 'welcome', 'announcements', 'off-topic'];
		const users = [
			{ name: 'Alex', avatar: '/placeholder.svg?height=40&width=40', color: 'bg-blue-500' },
			{ name: 'Taylor', avatar: '/placeholder.svg?height=40&width=40', color: 'bg-green-500' },
			{ name: 'Jordan', avatar: '/placeholder.svg?height=40&width=40', color: 'bg-purple-500' },
			{ name: 'Casey', avatar: '/placeholder.svg?height=40&width=40', color: 'bg-yellow-500' },
		];

		let mockMessages = [];
		const today = new Date();

		for (let i = 0; i < 15; i++) {
			const user = users[Math.floor(Math.random() * users.length)];
			const time = new Date(today);
			time.setMinutes(today.getMinutes() - i * 15);

			mockMessages.push({
				id: i,
				user,
				content: `This is a message in #${channelNames[activeChannel]} on the ${serverNames[activeServer]} server. Message #${i + 1}`,
				timestamp: time,
				reactions: i % 3 === 0 ? [{ emoji: '👍', count: Math.floor(Math.random() * 5) + 1 }] : [],
			});
		}
		messages = mockMessages.reverse();
	}

	function handleSendMessage(e: Event) {
		e.preventDefault();
		if (message.trim()) {
			const newMessage = {
				id: messages.length,
				user: {
					name: 'You',
					avatar: '/placeholder.svg?height=40&width=40',
					color: 'bg-[#5865f2]',
				},
				content: message,
				timestamp: new Date(),
				reactions: [],
			};

			messages = [...messages, newMessage];
			message = '';

			// Scroll to bottom after a slight delay to allow the DOM to update.
			setTimeout(() => {
				if (scrollAreaRef) {
					const scrollContainer = scrollAreaRef.querySelector('[data-radix-scroll-area-viewport]');
					if (scrollContainer) {
						scrollContainer.scrollTop = scrollContainer.scrollHeight;
					}
				}
			}, 100);
		}
	}

	// Group messages by consecutive messages from the same user.
	$: groupedMessages = messages.reduce((acc: any[], msg, index) => {
		if (index === 0 || messages[index - 1].user.name !== msg.user.name) {
			acc.push({
				user: msg.user,
				messages: [
					{ id: msg.id, content: msg.content, timestamp: msg.timestamp, reactions: msg.reactions },
				],
			});
		} else {
			acc[acc.length - 1].messages.push({
				id: msg.id,
				content: msg.content,
				timestamp: msg.timestamp,
				reactions: msg.reactions,
			});
		}
		return acc;
	}, []);

	$: {
		console.log(JSON.stringify(groupedMessages));
	}

	const formatTime = (date: Date) => {
		return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	};

	const formatDate = (date: Date) => {
		const today = new Date();
		const yesterday = new Date(today);
		yesterday.setDate(yesterday.getDate() - 1);

		if (date.toDateString() === today.toDateString()) {
			return 'Today';
		} else if (date.toDateString() === yesterday.toDateString()) {
			return 'Yesterday';
		} else {
			return date.toLocaleDateString();
		}
	};

	// Determine whether to show a date divider before a group.
	const shouldShowDateDivider = (currentIndex: number, currentDate: Date) => {
		if (currentIndex === 0) return true;
		const prevMessage = groupedMessages[currentIndex - 1].messages[0];
		const prevDate = new Date(prevMessage.timestamp);
		return prevDate.toDateString() !== currentDate.toDateString();
	};
</script>

<div class="flex min-w-0 flex-1 flex-col bg-[#36393f]">
	<!-- Channel Header -->
	<div class="flex h-12 items-center border-b border-[#202225] px-4 shadow-sm">
		<Hash class="mr-2 h-6 w-6 text-gray-400" />
		<h3 class="font-semibold text-white">
			{#if activeChannel >= 0}
				{['general', 'welcome', 'announcements', 'off-topic'][activeChannel]}
			{/if}
		</h3>
		<div class="mx-2 h-6 border-l border-gray-600"></div>
		<p class="truncate text-sm text-gray-400">
			Welcome to the channel! This is the start of the channel.
		</p>
		<div class="ml-auto flex items-center space-x-4">
			<button class="text-gray-400 hover:text-gray-200">
				<Bell class="h-5 w-5" />
			</button>
			<button class="text-gray-400 hover:text-gray-200">
				<PinIcon class="h-5 w-5" />
			</button>
			<button class="text-gray-400 hover:text-gray-200">
				<Users class="h-5 w-5" />
			</button>
		</div>
	</div>

	<!-- Messages Area -->
	<ScrollArea class="flex-1" bind:this={scrollAreaRef}>
		<div class="space-y-4 p-4">
			{#each groupedMessages as group, groupIndex}
				<div>
					{#if shouldShowDateDivider(groupIndex, new Date(group.messages[0].timestamp))}
						<div class="my-4 flex items-center">
							<div class="h-[1px] flex-1 bg-gray-600"></div>
							<span class="px-4 text-xs font-medium text-gray-400">
								{formatDate(new Date(group.messages[0].timestamp))}
							</span>
							<div class="h-[1px] flex-1 bg-gray-600"></div>
						</div>
					{/if}

					<div class="group flex">
						<Avatar class="mr-4 mt-0.5 h-10 w-10">
							<AvatarImage src={group.user.avatar} />
							<AvatarFallback class={group.user.color}>
								{group.user.name.charAt(0)}
							</AvatarFallback>
						</Avatar>
						<div class="min-w-0 flex-1">
							<div class="flex items-center">
								<span class="font-medium text-white">{group.user.name}</span>
								<span class="ml-2 text-xs text-gray-400">
									{formatTime(new Date(group.messages[0].timestamp))}
								</span>
							</div>
							<div class="mt-1 space-y-1">
								{#each group.messages as msg}
									<div>
										<p class="text-gray-300">{msg.content}</p>
										{#if msg.reactions.length > 0}
											<div class="mt-1 flex space-x-1">
												{#each msg.reactions as reaction, i}
													<div
														class="flex cursor-pointer items-center rounded-full bg-[#2f3136] px-2 py-0.5 hover:bg-[#393c43]"
													>
														<span class="mr-1">{reaction.emoji}</span>
														<span class="text-xs text-gray-300">{reaction.count}</span>
													</div>
												{/each}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					</div>
				</div>
			{/each}
		</div>
	</ScrollArea>

	<!-- Message Input -->
	<div class="px-4 pb-6 pt-2">
		<form on:submit|preventDefault={handleSendMessage} class="relative">
			<div class="flex items-center rounded-lg bg-[#40444b] px-4">
				<button type="button" class="mr-2 text-gray-400 hover:text-gray-200">
					<PlusCircle class="h-5 w-5" />
				</button>
				<Input
					bind:value={message}
					placeholder={`Message #${['general', 'welcome', 'announcements', 'off-topic'][activeChannel]}`}
					class="flex-1 border-none bg-transparent text-gray-200 placeholder:text-gray-400 focus-visible:ring-0 focus-visible:ring-offset-0"
				/>
				<div class="ml-2 flex items-center space-x-2">
					<button type="button" class="text-gray-400 hover:text-gray-200">
						<Gift class="h-5 w-5" />
					</button>
					<button type="button" class="text-gray-400 hover:text-gray-200">
						<GIF class="h-5 w-5" />
					</button>
					<button type="button" class="text-gray-400 hover:text-gray-200">
						<Paperclip class="h-5 w-5" />
					</button>
					<button type="button" class="text-gray-400 hover:text-gray-200">
						<Smile class="h-5 w-5" />
					</button>
					{#if message.trim()}
						<Button
							type="submit"
							size="icon"
							variant="ghost"
							class="p-0 text-gray-200 hover:bg-transparent"
						>
							<Send class="h-5 w-5" />
						</Button>
					{/if}
				</div>
			</div>
		</form>
	</div>
</div>
