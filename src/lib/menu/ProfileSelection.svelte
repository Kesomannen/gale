<script lang="ts">
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { activeProfile, profiles, setActiveProfile } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu } from 'bits-ui';

	export let onSelect: () => void;

	let searchTerm: string;
</script>

<div class="mt-1 flex flex-col">
	<div class="relative flex text-white">
		<div class="relative flex-grow">
			<SearchBar bind:value={searchTerm} placeholder="Search in your profiles..." />
		</div>

		<Button.Root
			class="ml-1 flex items-center gap-2 rounded-l-lg bg-accent-700 px-4 hover:bg-accent-600"
		>
			<Icon icon="mdi:plus" class="text-lg" />
			Create
		</Button.Root>

		<DropdownMenu.Root>
			<DropdownMenu.Trigger class="ml-0.5 rounded-r-lg bg-accent-700 px-1 hover:bg-accent-600">
				<Icon icon="mdi:expand-more" class="text-lg" />
			</DropdownMenu.Trigger>
		</DropdownMenu.Root>
	</div>

	<div class="mt-2 flex h-72 flex-col overflow-y-scroll">
		{#each profiles as profile, i}
			<Button.Root
				class="group mr-2 flex items-center rounded-lg border border-slate-500 p-1.5 hover:bg-slate-700 {profile ==
				$activeProfile
					? 'bg-slate-700'
					: 'border-opacity-0 hover:bg-slate-700'}"
				on:click={() => {
					setActiveProfile(i);
					onSelect();
				}}
			>
				<div class="flex-grow pl-1 text-left">
					<div class="font-medium text-white">
						{profile.name}
					</div>
				</div>
			</Button.Root>
		{/each}
	</div>
</div>
