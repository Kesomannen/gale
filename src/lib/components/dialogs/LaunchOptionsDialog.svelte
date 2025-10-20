<script lang="ts">
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { RadioGroup } from 'bits-ui';
	import type { LaunchOption } from '$lib/types';
	import { formatLaunchOptionName } from '$lib/util';

	interface Props {
		open: boolean;
		options: LaunchOption[];
		gameName: string;
		onselect: (args: string) => void;
	}

	let { open = $bindable(), options, gameName, onselect }: Props = $props();
	let selectedOption = $state<string>(options[0]?.arguments ?? '');

	function launch() {
		open = false;
		onselect(selectedOption);
	}

	function handleCancel() {
		open = false;
	}

	$effect(() => {
		if (options.length > 0 && !options.find((o) => o.arguments === selectedOption)) {
			selectedOption = options[0].arguments;
		}
	});
</script>

<ConfirmDialog bind:open title="Launch {gameName}" onCancel={handleCancel}>
	<p class="text-primary-400 mb-4">Select how you want to launch the game:</p>

	<div class="max-h-80 overflow-y-auto">
		<RadioGroup.Root bind:value={selectedOption} class="flex flex-col gap-1">
			{#each options as option}
				<RadioGroup.Item
					value={option.arguments}
					class={[
						'flex cursor-pointer items-center rounded-lg border p-3',
						selectedOption === option.arguments
							? 'border-primary-500 bg-primary-700'
							: 'hover:bg-primary-700 border-transparent'
					]}
				>
					<div class="mr-3 flex h-5 w-5 items-center justify-center">
						<div
							class={[
								'flex h-4 w-4 items-center justify-center rounded-full border-2',
								selectedOption === option.arguments ? 'border-accent-400' : 'border-primary-400'
							]}
						>
							{#if selectedOption === option.arguments}
								<div class="bg-accent-400 h-2 w-2 rounded-full"></div>
							{/if}
						</div>
					</div>
					<div class="flex text-left">
						<div class="font-medium text-white">
							{formatLaunchOptionName(option.type, gameName, option.description)}
						</div>
					</div>
				</RadioGroup.Item>
			{/each}
		</RadioGroup.Root>
	</div>

	<div class="text-primary-400 mt-4 text-xs">
		You can disable this dialog by turning off "Show Steam launch options" in <a
			href="/prefs"
			onclick={() => (open = false)}
			class="text-primary-400 hover:text-primary-300 underline"
		>
			Settings</a
		>
	</div>

	{#snippet buttons()}
		<Button icon="mdi:play-circle" onclick={launch}>Launch</Button>
	{/snippet}
</ConfirmDialog>
