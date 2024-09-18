<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';

	import Icon from '@iconify/svelte';
	import { Dialog, Popover, Progress } from 'bits-ui';
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Updater from './Updater.svelte';
	import { games, profiles } from '$lib/state/profile.svelte';
	import { invoke } from '$lib/invoke';
	import loadingBars from '$lib/state/loading.svelte';
	import { fly, slide } from 'svelte/transition';

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
		{#if games.active !== undefined}
			<img
				src="games/{games.active.slug}.webp"
				class="size-8 rounded mr-2"
				alt={games.active.name}
			/>

			{games.active.name}
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

	<Popover.Root>
		<Popover.Trigger class="size-8 my-auto rounded bg-green-600 hover:bg-green-500 text-white">
			<Icon icon="mdi:loading" class="text-xl animate-spin mx-auto" />
		</Popover.Trigger>
		<Popover.Content class="flex flex-col px-4 py-2 border border-gray-600 rounded-lg shadow-lg bg-gray-800 w-96"
		inTransition={fly}
		inTransitionConfig={{ duration: 50 }}>
			{#each loadingBars.all as { id, title, message, progress }}
				<div>
					<Progress.Root
					value={progress ?? 1}
					max={1}
					class="relative h-4 mt-2 bg-gray-900 rounded-full overflow-hidden"
				>
					<div
						class="absolute top-0 left-0 h-full bg-green-600 rounded-l-full transition-all"
						style="width: {(progress ?? 1) * 100}%"
					></div>
				</Progress.Root>

				<h3 class="text-white font-medium pt-2">{title}</h3>
				<h4 class="text-gray-400 text-sm">{message}</h4>
				</div>
			{/each}
		</Popover.Content>
	</Popover.Root>

	<Updater />
</div>

<Popup title="Launching {games.active?.name}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-gray-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onselect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
