<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';

	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Updater from './Updater.svelte';
	import { communities, profiles } from '$lib/state/profile.svelte';
	import { invoke } from '$lib/invoke';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function launchGame() {
		invoke('profile', 'launch', { id: profiles.activeId })
		launchGamePopupOpen = true;
	}
</script>

<div class="h-12 flex flex-row flex-shrink-0 bg-gray-900 border-b border-t border-gray-600">
	<button
		class="flex items-center flex-shrink-0 pl-6 pr-8 border-r border-gray-600 text-green-400 hover:text-green-400 hover:bg-gray-800 font-bold"
		onclick={launchGame}
	>
		<Icon icon="material-symbols:play-circle" class="text-xl mr-2" />
		Launch game
	</button>

	<button
		onclick={() => (gamesOpen = !gamesOpen)}
		class="flex flex-shrink-0 items-center justify-between font-semibold pl-2 pr-4 group border-r border-gray-600 hover:bg-gray-800 text-gray-300 group-hover:text-gray-200"
	>
		{#if communities.active !== undefined}
			<img
				src="games/{communities.active.slug}.webp"
				class="size-8 rounded mr-2"
				alt={communities.active.name}
			/>

			{communities.active.name}
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="text-gray-300 group-hover:text-gray-200 text-xl transition-all flex-shrink-0 ml-6"
		/>
	</button>

	<button
		class="flex flex-shrink items-center min-w-40 pl-6 pr-4 group border-r border-gray-600 text-gray-300 group-hover:text-gray-200 hover:bg-gray-800"
	>
		{#if profiles.active !== undefined}
			<span class="flex-shrink truncate font-semibold mr-auto">
				{profiles.active.name}
			</span>

			<div
				class="rounded bg-gray-800 group-hover:bg-gray-700 px-2 py-0.5 text-sm ml-6 mr-2 font-medium"
			>
				{profiles.active.mods.length}
			</div>
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="flex-shrink-0 text-xl transition-all transform origin-center {profilesOpen
				? 'rotate-180'
				: 'rotate-0'}"
		/>
	</button>

	<Updater />
</div>

<Popup title="Launching {communities.active?.name}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-gray-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onselect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
