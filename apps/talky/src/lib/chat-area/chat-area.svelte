<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Separator } from '$lib/components/ui/separator';
	import UserList from '$lib/user-list/user-list.svelte';
	import { Send, Paperclip, AtSign, Smile } from 'lucide-svelte';

	type Message = {
		id: string;
		content: string;
		author: {
			name: string;
			avatar: string;
		};
		timestamp: string;
		isSystem?: boolean;
	};

	const state = $state({
		inputValue: ''
	});

	const messages: Message[] = [
		{
			id: '1',
			content: '*** Game Day Thread: Devils vs. Flyers - April 4, 2025 ***',
			author: {
				name: 'System',
				avatar: ''
			},
			timestamp: '12:00 PM',
			isSystem: true
		},
		{
			id: '2',
			content: 'Anyone heading to the game tonight?',
			author: {
				name: 'DevilsFan83',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:05 PM'
		},
		{
			id: '3',
			content: "I'll be there! Section 122, row 8.",
			author: {
				name: 'JerseyDevil',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:07 PM'
		},
		{
			id: '4',
			content: "Hoping our top line shows up tonight. They've been quiet the last few games.",
			author: {
				name: 'HockeyAnalyst',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:10 PM'
		},
		{
			id: '5',
			content: 'Lineup just posted - looks like Smith is back from injury!',
			author: {
				name: 'DevilsDaily',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:15 PM'
		},
		{
			id: '6',
			content: "That's huge news! Our defense has been struggling without him.",
			author: {
				name: 'JerseyDevil',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:17 PM'
		},
		{
			id: '7',
			content: 'Prediction: Devils 3, Flyers 1',
			author: {
				name: 'DevilsFan83',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:20 PM'
		},
		{
			id: '8',
			content: "I'm going with 4-2 Devils. Hughes with 2 goals tonight.",
			author: {
				name: 'HockeyAnalyst',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '12:22 PM'
		}
	];
</script>

<div class="flex h-full w-full">
	<div class="flex h-full flex-1 flex-col overflow-y-auto">
		<div class="mt-auto space-y-4 p-4">
			{#each messages as message}
				<div class={message.isSystem ? 'text-center' : ''}>
					{#if message.isSystem}
						<div class="inline-block rounded-md bg-primary px-3 py-1 text-primary-foreground">
							<p class="font-mono text-xs">{message.content}</p>
						</div>
					{:else}
						<div class="flex items-start">
							<Avatar class="mr-2 mt-1 h-8 w-8">
								<AvatarImage src={message.author.avatar} alt={message.author.name} />
								<AvatarFallback class="bg-foreground text-xs text-background">
									{message.author.name.substring(0, 2).toUpperCase()}
								</AvatarFallback>
							</Avatar>
							<div class="flex-1">
								<div class="flex items-baseline">
									<span class="font-mono font-bold text-foreground/60">{message.author.name}</span>
									<span class="ml-2 font-mono">{message.timestamp}</span>
								</div>
								<p class="font-mono text-sm">{message.content}</p>
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<div class="p-2">
			<div class="rounded bg-primary/10 p-2">
				<div class=" flex items-center">
					<Button variant="ghost" size="icon" class="h-8 w-8  hover:text-primary">
						<Paperclip class="h-4 w-4" />
					</Button>
					<div class="relative mx-2 flex-1">
						<Input
							class=" border-transparent bg-transparent font-mono"
							placeholder="Message #gameday"
							bind:value={state.inputValue}
						/>
						<div
							class="absolute right-2 top-1/2 flex -translate-y-1/2 transform items-center space-x-1"
						>
							<Button variant="ghost" size="icon" class="h-6 w-6  hover:text-primary">
								<AtSign class="h-4 w-4" />
							</Button>
							<Button variant="ghost" size="icon" class="h-6 w-6  hover:text-primary">
								<Smile class="h-4 w-4" />
							</Button>
						</div>
					</div>
					<Button
						size="icon"
						class="h-8 w-8 bg-primary text-primary-foreground/60 hover:text-primary-foreground"
					>
						<Send class="h-4 w-4" />
					</Button>
				</div>
			</div>
		</div>
	</div>
	<div>
		<UserList />
	</div>
</div>
