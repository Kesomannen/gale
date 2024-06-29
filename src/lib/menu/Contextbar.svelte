<script lang="ts">
	import { confirm } from '@tauri-apps/api/dialog';

	import Popup from '$lib/components/Popup.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';

	import {
		activeProfileIndex,
		activeGame,
		activeProfile,
		profiles,
		refreshProfiles,
		setActiveProfile
	} from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';

	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Updater from './Updater.svelte';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profiles[index].name}?`, {
			title: 'Delete profile'
		}).then(async (result) => {
			if (result) {
				await invokeCommand('delete_profile', { index });
				refreshProfiles();
			}
		});
	}
</script>

<div class="h-12 flex flex-row flex-shrink-0 bg-gray-900 border-b border-t border-gray-600">
	<Button.Root
		class="flex items-center flex-shrink-0 pl-6 pr-8 border-r border-gray-600 text-green-400 hover:text-green-400 hover:bg-gray-800 cursor-default"
		on:click={() => {
			invokeCommand('launch_game');
			launchGamePopupOpen = true;
		}}
	>
		<Icon icon="mdi:play-circle" class="text-xl mr-2" />
		<div class="font-semibold">Launch game</div>
	</Button.Root>

	<Button.Root
		on:click={() => (gamesOpen = !gamesOpen)}
		class="flex flex-shrink-0 items-center justify-between font-semibold pl-2 pr-4 group border-r border-gray-600 hover:bg-gray-800 text-slate-300 group-hover:text-slate-200 cursor-default"
	>
		{#if $activeGame}
			<img
				src="games/{$activeGame.id}.webp"
				class="max-w-8 max-h-8 rounded mr-2"
				alt={$activeGame.displayName}
			/>

			{$activeGame.displayName}
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="text-slate-300 group-hover:text-slate-200 text-xl transition-all flex-shrink-0 ml-6"
		/>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="flex flex-shrink items-center min-w-40 pl-6 pr-4 group border-r border-gray-600 
						text-slate-300 group-hover:text-slate-200 hover:bg-gray-800 cursor-default"
		>
			<span class="flex-shrink truncate font-semibold mr-auto">
				{$activeProfile?.name}
			</span>

			<div
				class="rounded bg-gray-800 group-hover:bg-gray-700 px-2 py-0.5 text-sm ml-6 mr-2 font-medium"
			>
				{$activeProfile?.modCount}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="flex-shrink-0 text-xl transition-all transform origin-center {profilesOpen
					? 'rotate-180'
					: 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600 min-w-40"
			inTransition={fly}
			inTransitionConfig={{ duration: 50 }}
		>
			{#each profiles as profile, i}
				<DropdownMenu.Item
					class="flex items-center pl-3 pr-1 py-1 cursor-default hover:bg-gray-700 rounded-md text-left group
						{i == activeProfileIndex
						? 'font-medium text-slate-300 hover:text-slate-200'
						: 'text-slate-400 hover:text-slate-300'}"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="flex-grow mr-3">
						{profile.name}
					</span>

					<Icon
						icon="mdi:check"
						class="text-green-500 text-lg mx-2 {i !== activeProfileIndex && 'invisible'}"
					/>

					<div
						class="rounded bg-gray-700 group-hover:bg-gray-600 px-1.5 py-0.5 text-xs mr-1"
					>
						{profile.modCount}
					</div>

					<Button.Root
						class="text-slate-400 hover:bg-red-600 hover:text-red-200 p-1 rounded"
						on:click={(evt) => {
							evt.stopPropagation();
							deleteProfile(i);
							profilesOpen = false;
						}}
					>
						<Icon icon="mdi:delete" />
					</Button.Root>
				</DropdownMenu.Item>
			{/each}

			<DropdownMenu.Item
				class="flex items-center justify-center py-1 cursor-default rounded-md font-semibold text-white bg-green-600/60 hover:bg-green-600"
				on:click={() => (newProfilePopupOpen = true)}
			>
				<Icon icon="mdi:plus" class="text-xl mr-1" />
				New profile
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<Updater />
</div>

<Popup title="Launching {$activeGame?.displayName}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-slate-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onSelect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
