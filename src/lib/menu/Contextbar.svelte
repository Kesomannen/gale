<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';

	import { activeGame } from '$lib/stores.svelte';
	import * as api from '$lib/api';
	import Icon from '@iconify/svelte';
	import { Dialog } from 'bits-ui';
	import GameSelect from '$lib/menu/GameSelect.svelte';
	import Updater from './Updater.svelte';
	import Syncer from './Syncer.svelte';
	import ProfilesDropdown from './ProfilesDropdown.svelte';

	let launchGamePopupOpen = $state(false);
	let gamesOpen = $state(false);

	function launchGame() {
		api.profile.launch.launchGame();
		launchGamePopupOpen = true;
	}
</script>

<div class="border-primary-600 bg-primary-900 flex h-12 shrink-0 flex-row border-t border-b">
	<div
		class="text-accent-400 hover:text-accent-400 border-primary-600 hover:bg-primary-800 shrink-0 border-r pr-8 pl-6"
	>
		<button class="flex h-full items-center font-semibold" onclick={launchGame}>
			<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
			Launch game
		</button>
	</div>

	<button
		onclick={() => (gamesOpen = !gamesOpen)}
		class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink-0 items-center justify-between border-r pr-4 pl-2 font-semibold"
	>
		<img
			src="games/{$activeGame?.slug}.webp"
			class="mr-2 max-h-8 max-w-8 rounded-sm"
			alt={$activeGame?.name}
		/>

		{$activeGame?.name}

		<Icon
			icon="mdi:menu"
			class="text-primary-300 group-hover:text-primary-200 ml-6 shrink-0 text-lg"
		/>
	</button>

	<ProfilesDropdown />
	<Syncer />
	<Updater />
</div>

<Popup title="Launching {$activeGame?.name}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-primary-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelect onselect={() => (gamesOpen = false)} />
</Popup>
