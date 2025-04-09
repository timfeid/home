<script>
	import * as Select from '$lib/components/ui/select';
	import { Label } from '$lib/components/ui/label';
	import { Button } from '$lib/components/ui/button';
	import { Slider } from '$lib/components/ui/slider';
	import { Switch } from '$lib/components/ui/switch';
	import { Mic, Volume2, VolumeX } from 'lucide-svelte';

	let inputVolume = $state(75);
	let outputVolume = $state(80);
	let noiseSuppression = $state(true);
	let echoCancellation = $state(true);
	let autoGain = $state(true);
</script>

<div class="flex h-full flex-col space-y-3">
	<div class="p-4">
		<h3 class="mb-2 text-lg font-semibold">Voice Settings</h3>
		<p class="mb-4 text-sm text-[#b5bac1]">Configure your input and output devices.</p>
	</div>

	<div class="flex-grow space-y-4 p-4">
		<div class="space-y-2">
			<Label for="input-device" class="text-xs font-semibold uppercase text-[#b5bac1]">
				Input Device
			</Label>
			<Select.Root type="single">
				<Select.Trigger class="w-full"></Select.Trigger>
				<Select.Content>
					<Select.Item value="default">Default - Microphone (Realtek Audio)</Select.Item>
					<Select.Item value="headset">Headset Microphone (USB Audio)</Select.Item>
					<Select.Item value="webcam">Webcam Microphone</Select.Item>
				</Select.Content>
			</Select.Root>
		</div>

		<div class="space-y-2">
			<Label for="output-device" class="text-xs font-semibold uppercase text-[#b5bac1]">
				Output Device
			</Label>
			<Select.Root type="single">
				<Select.Trigger class="w-full"></Select.Trigger>
				<Select.Content>
					<Select.Item value="default">Default - Microphone (Realtek Audio)</Select.Item>
					<Select.Item value="headset">Headset Microphone (USB Audio)</Select.Item>
					<Select.Item value="webcam">Webcam Microphone</Select.Item>
				</Select.Content>
			</Select.Root>
		</div>

		<div class="space-y-2 pt-4">
			<div class="flex items-center justify-between">
				<Label for="input-volume" class="flex items-center gap-2">
					<Mic class="h-4 w-4" />
					<span>Input Volume</span>
				</Label>
				<span class="text-xs text-[#b5bac1]">{inputVolume}%</span>
			</div>
			<Slider type="single" max={100} min={0} step={1} />
			<div class="flex justify-between text-xs text-[#b5bac1]">
				<VolumeX class="h-3 w-3" />
				<Volume2 class="h-3 w-3" />
			</div>
		</div>

		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<Label for="output-volume" class="flex items-center gap-2">
					<Volume2 class="h-4 w-4" />
					<span>Output Volume</span>
				</Label>
				<span class="text-xs text-[#b5bac1]">{outputVolume}%</span>
			</div>
			<Slider type="multiple" max={100} min={0} step={1} />
			<div class="flex justify-between text-xs text-[#b5bac1]">
				<VolumeX class="h-3 w-3" />
				<Volume2 class="h-3 w-3" />
			</div>
		</div>

		<div class="space-y-4 pt-4">
			<h4 class="font-medium">Voice Processing</h4>

			<div class="flex items-center justify-between">
				<div>
					<Label for="noise-suppression" class="font-medium">Noise Suppression</Label>
					<p class="text-sm text-[#b5bac1]">Reduces background noise when you speak</p>
				</div>
				<Switch bind:checked={noiseSuppression} />
			</div>

			<div class="flex items-center justify-between">
				<div>
					<Label for="echo-cancellation" class="font-medium">Echo Cancellation</Label>
					<p class="text-sm text-[#b5bac1]">Prevents echo from speakers</p>
				</div>
				<Switch bind:checked={echoCancellation} />
			</div>

			<div class="flex items-center justify-between">
				<div>
					<Label for="auto-gain" class="font-medium">Automatic Gain Control</Label>
					<p class="text-sm text-[#b5bac1]">Automatically adjusts microphone volume</p>
				</div>
				<Switch bind:checked={autoGain} />
			</div>
		</div>
	</div>

	<div class="sticky bottom-0 mt-auto flex border-t border-[#3f4147] bg-background p-4">
		<Button class="ml-auto">Save Changes</Button>
	</div>
</div>
