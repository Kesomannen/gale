<script lang="ts">
	import { confirm } from '@tauri-apps/api/dialog';

	import Popup from '$lib/Popup.svelte';
	import {
		activeProfileIndex,
		currentGame,
		currentProfile,
		games,
		profileNames,
		refreshProfiles,
		setActiveGame,
		setActiveProfile
	} from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';
	
	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';
	import { sentenceCase } from '$lib/util';

	let gamesOpen = false;
	let profilesOpen = false;
	let startingGame = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profileNames[index]}?`, {
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
		class="flex flex-row items-center pl-6 pr-8 border-r border-gray-600 text-green-400 hover:text-green-400 hover:bg-green-900"
		on:click={() => {
			invokeCommand('start_game').then(() => (startingGame = true));
		}}
	>
		<Icon icon="mdi:play-circle" class="text-xl mr-2" />
		<div class="font-medium truncate">Run game</div>
	</Button.Root>

	<DropdownMenu.Root bind:open={gamesOpen}>
		<DropdownMenu.Trigger
			class="flex items-center justify-between gap-2 pl-2 pr-4 group border-r border-gray-600 hover:bg-gray-800 cursor-default"
		>
			<img src="games/{$currentGame.name}.png" class="max-w-8 max-h-8 rounded" alt={$currentGame.displayName} />

			<div class="text-slate-300 group-hover:text-slate-200 truncate">
				{$currentGame.displayName}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="
                text-slate-300 group-hover:text-slate-200 text-xl transition-all flex-shrink-0
                transform origin-center {gamesOpen ? 'rotate-180' : 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-52 rounded-lg border border-gray-600"
		>
			{#each games as game}
				<DropdownMenu.Item
					class="flex items-center px-1 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
					on:click={() => {
						setActiveGame(game);
						profilesOpen = false;
					}}
				>
					<img src="games/{game.name}.png" class="max-w-8 max-h-8 rounded mr-2" alt={game.displayName} />

					<span class="flex-shrink truncate">
						{game.displayName}
					</span>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="flex items-center justify-between gap-2 w-40 pl-6 pr-4 group border-r border-gray-600 relative hover:bg-gray-800 cursor-default"
		>
			<div class="text-slate-300 group-hover:text-slate-200 flex-shrink truncate">
				{$currentProfile}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="
                text-slate-300 group-hover:text-slate-200 text-xl transition-all
                transform origin-center {profilesOpen ? 'rotate-180' : 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-40 rounded-lg border border-gray-600"
		>
			{#each profileNames as profile, i}
				<DropdownMenu.Item
					class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="flex-shrink truncate">
						{profile}
					</span>

					<div class="ml-auto inline-flex">
						{#if i == activeProfileIndex}
							<Icon icon="mdi:check" class=" text-green-400 text-lg" />
						{/if}
						<Button.Root class="text-red-500 hover:text-red-400 text-lg ml-1" on:click={(evt) => {
								evt.stopPropagation();
								deleteProfile(i);
							}}>
							<Icon icon="mdi:delete" />
						</Button.Root>
					</div>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>

<Popup title="Starting game..." bind:open={startingGame}>
	<Dialog.Description class="text-slate-400">
		Click outside this window to continue modding. If it's taking a while, it's probably because
		Steam is starting up.
	</Dialog.Description>
</Popup>