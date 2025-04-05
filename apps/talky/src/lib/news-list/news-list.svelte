<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { ThumbsUp, MessageSquare, Share2, Bookmark, MoreHorizontal } from 'lucide-svelte';

	type NewsItem = {
		id: string;
		title: string;
		content: string;
		author: {
			name: string;
			avatar: string;
		};
		timestamp: string;
		upvotes: number;
		comments: number;
		tags: string[];
	};

	const newsItems: NewsItem[] = [
		{
			id: '1',
			title: 'Devils Sign Top Prospect to Entry-Level Contract',
			content:
				"The New Jersey Devils have signed their top prospect to a three-year entry-level contract. The 19-year-old forward was the team's first-round pick in last year's draft and has been impressing scouts with his performance in the junior leagues this season.",
			author: {
				name: 'HockeyInsider',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '2h ago',
			upvotes: 128,
			comments: 43,
			tags: ['Signing', 'Prospects']
		},
		{
			id: '2',
			title: 'Injury Update: Star Forward Out 2-3 Weeks',
			content:
				"The Devils will be without their star forward for the next 2-3 weeks due to a lower-body injury sustained in last night's game against the Rangers. The team has called up a promising young player from their AHL affiliate to fill the roster spot.",
			author: {
				name: 'DevilsDaily',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '5h ago',
			upvotes: 95,
			comments: 67,
			tags: ['Injury', 'Roster']
		},
		{
			id: '3',
			title: 'Game Preview: Devils vs. Flyers - Rivalry Night',
			content:
				'The Devils host the Philadelphia Flyers tonight in what promises to be an intense divisional matchup. Both teams are fighting for playoff positioning, and this game could have significant implications for the standings. Puck drop is scheduled for 7:00 PM ET.',
			author: {
				name: 'PuckReport',
				avatar: '/placeholder.svg?height=40&width=40'
			},
			timestamp: '8h ago',
			upvotes: 72,
			comments: 31,
			tags: ['Game Preview', 'Rivalry']
		}
	];
</script>

<div class="max-w-4xl px-3 py-3">
	<div class="space-y-3">
		{#each newsItems as item}
			<div class="overflow-hidden rounded-md border-t dark:bg-foreground/5">
				<div class="p-3">
					<div class="mb-3 flex items-center">
						<Avatar class="mr-2 h-8 w-8">
							<AvatarImage src={item.author.avatar} alt={item.author.name} />
							<AvatarFallback class="bg-primary text-xs text-primary-foreground ">
								{item.author.name.substring(0, 2).toUpperCase()}
							</AvatarFallback>
						</Avatar>
						<div>
							<p class="text-sm font-medium">{item.author.name}</p>
							<p class="text-xs">{item.timestamp}</p>
						</div>
						<Button variant="ghost" size="icon" class="ml-auto h-8 w-8">
							<MoreHorizontal class="h-4 w-4" />
						</Button>
					</div>

					<h3 class="mb-2 text-lg font-bold">{item.title}</h3>
					<p class="mb-3 text-sm">{item.content}</p>

					<div class="mb-3 flex flex-wrap gap-2">
						{#each item.tags as tag}
							<span class="rounded bg-primary px-2 py-0.5 text-xs text-primary-foreground"
								>{tag}</span
							>
						{/each}
					</div>

					<div class="flex items-center justify-between pt-2">
						<div class="flex items-center space-x-4">
							<button class="hover:primary/5 flex items-center transition-colors">
								<ThumbsUp class="mr-1 h-4 w-4" />
								<span class="text-xs">{item.upvotes}</span>
							</button>
							<button class="hover:primary/5 flex items-center transition-colors">
								<MessageSquare class="mr-1 h-4 w-4" />
								<span class="text-xs">{item.comments}</span>
							</button>
						</div>
						<div class="flex items-center space-x-2">
							<Button variant="ghost" size="icon" class="hover:primary/5 h-8  w-8">
								<Bookmark class="h-4 w-4" />
							</Button>
							<Button variant="ghost" size="icon" class="hover:primary/5 h-8  w-8">
								<Share2 class="h-4 w-4" />
							</Button>
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
