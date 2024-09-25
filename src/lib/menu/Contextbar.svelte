<script context="module" lang="ts">
</script>

<script lang="ts">
	import { confirm } from '@tauri-apps/plugin-dialog';

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

	import { t, T } from '$i18n';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function deleteProfile(index: number) {
		confirm(T('Delete profile description', {"name": profiles[index].name}), {
			title: t('Delete profile')
		}).then(async (result) => {
			if (result) {
				await invokeCommand('delete_profile', { index });
				refreshProfiles();
			}
		});
	}

	function launchGame(vanilla: boolean) {
		invokeCommand('launch_game', { vanilla });
		launchGamePopupOpen = true;
	}
</script>

<div class="flex h-12 flex-shrink-0 flex-row border-b border-t border-gray-600 bg-gray-900">
	<div
		class="flex-shrink-0 border-r border-gray-600 pl-6 pr-8 text-green-400 hover:bg-gray-800 hover:text-green-400"
	>
		<Button.Root
			class="flex h-full cursor-default items-center font-semibold"
			on:click={() => launchGame(false)}
		>
			<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
			{t('Launch game')}
		</Button.Root>
	</div>

	<Button.Root
		on:click={() => (gamesOpen = !gamesOpen)}
		class="group flex flex-shrink-0 cursor-default items-center justify-between border-r border-gray-600 pl-2 pr-4 font-semibold text-slate-300 hover:bg-gray-800 group-hover:text-slate-200"
	>
		{#if $activeGame}
			<img
				src="games/{$activeGame.id}.webp"
				class="mr-2 max-h-8 max-w-8 rounded"
				alt={$activeGame.displayName}
			/>

			{$activeGame.displayName}
		{:else}
			{t("Loading")}
		{/if}

		<Icon
			icon="mdi:menu"
			class="ml-6 flex-shrink-0 text-xl text-slate-300 transition-all group-hover:text-slate-200"
		/>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="group flex min-w-40 flex-shrink cursor-default items-center border-r border-gray-600 pl-6 
						pr-4 text-slate-300 hover:bg-gray-800 group-hover:text-slate-200"
		>
			<span class="mr-auto flex-shrink truncate font-semibold">
				{$activeProfile?.name}
			</span>

			<div
				class="ml-6 mr-2 rounded bg-gray-800 px-2 py-0.5 text-sm font-medium group-hover:bg-gray-700"
			>
				{$activeProfile?.modCount}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="flex-shrink-0 origin-center transform text-xl transition-all {profilesOpen
					? 'rotate-180'
					: 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-lg border border-gray-600 bg-gray-800 p-1 shadow-xl"
			inTransition={fly}
			inTransitionConfig={{ duration: 50 }}
		>
			{#each profiles as profile, i}
				<DropdownMenu.Item
					class="group flex cursor-default items-center rounded-md py-1 pl-3 pr-1 text-left hover:bg-gray-700
						{i == activeProfileIndex
						? 'font-medium text-slate-300 hover:text-slate-200'
						: 'text-slate-400 hover:text-slate-300'}"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="mr-3 flex-grow">
						{profile.name}
					</span>

					<Icon
						icon="mdi:check"
						class="mx-2 text-lg text-green-500 {i !== activeProfileIndex && 'invisible'}"
					/>

					<div class="mr-1 rounded bg-gray-700 px-1.5 py-0.5 text-xs group-hover:bg-gray-600">
						{profile.modCount}
					</div>

					<Button.Root
						class="rounded p-1 text-slate-400 hover:bg-red-600 hover:text-red-200"
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
				class="flex cursor-default items-center justify-center rounded bg-green-700 py-1 text-white hover:bg-green-600"
				on:click={() => (newProfilePopupOpen = true)}
			>
				<Icon icon="mdi:plus" class="mr-1 text-xl" />
				{t('New profile')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<Updater />
</div>

<Popup title="Launching {$activeGame?.displayName}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-slate-400">
		{t('Launch game description')}
	</Dialog.Description>
</Popup>

<Popup title="{t('Select game to mod')}" bind:open={gamesOpen}>
	<GameSelection onSelect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
