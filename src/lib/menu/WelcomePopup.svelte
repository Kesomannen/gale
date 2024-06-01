<script lang="ts">
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import BigButton from '$lib/components/BigButton.svelte';

	import LaunchModePref from '$lib/prefs/LaunchModePref.svelte';
	import ZoomLevelPref from '$lib/prefs/ZoomFactorPref.svelte';
	import PathSetting from '$lib/prefs/PathPref.svelte';

	import { refreshProfiles } from '$lib/stores';
	import type { R2ImportData } from '$lib/models';

	import { invokeCommand } from '$lib/invoke';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import ImportR2Flow from '$lib/import/ImportR2Flow.svelte';

	export let open = false;

	let stage: 'gameSelect' | 'importProfiles' | 'settings' | 'end' = 'gameSelect';

	let importFrom: 'r2modman' | 'thunderstore' = 'r2modman';
	let importData: R2ImportData = {
		r2modman: undefined,
		thunderstore: undefined
	};

	let importFlow: ImportR2Flow;

	$: title = stage === 'importProfiles' ? 'Import profiles' : 'Welcome to Gale!';
	$: importText =
		importData.r2modman && importData.thunderstore
			? 'r2modman or Thunderstore Mod Manager'
			: importData.r2modman
				? 'r2modman'
				: 'Thunderstore Mod Manager';

	onMount(async () => {
		if (await invokeCommand<boolean>('is_first_run')) {
			open = true;
		}
	});

	async function onSelectGame() {
		let result = await invokeCommand<R2ImportData>('get_r2modman_info');

		if (!result.r2modman && !result.thunderstore) {
			stage = 'settings';
			return;
		}

		importData = result;

		if (result.r2modman) {
			result.r2modman.include = result.r2modman.profiles.map(() => true);
			importFrom = 'r2modman';
		}

		if (result.thunderstore) {
			result.thunderstore.include = result.thunderstore.profiles.map(() => true);
			importFrom = 'thunderstore';
		}

		stage = 'importProfiles';
	}

	async function importProfiles() {
		await importFlow.doImport();
		stage = 'settings';
	}
</script>

<Popup {title} canClose={stage === 'end'} bind:open maxWidth="[55%]">
	<div class="text-slate-300">
		{#if stage === 'gameSelect'}
			To get started, select a game to mod:
			<GameSelection onSelect={onSelectGame} />
		{:else if stage === 'importProfiles' && importData}
			<p>
				You can automatically transfer profiles from {importText} to Gale.
			</p>

			<p class="mt-1">
				The process may take a couple of minutes, depending on how many mods and profiles there are
				to import. It will also transfer configs and cached mods.
			</p>

			<p class="mt-1">
				You can always import profiles later by going to <b>Import > ...from r2modman</b>.
			</p>

			<ImportR2Flow bind:importData bind:importFrom bind:this={importFlow} />

			<div class="flex mt-2 gap-1.5">
				<BigButton color="gray" onClick={() => (stage = 'gameSelect')}>Back</BigButton>
				<div class="flex-grow" />
				<BigButton color="gray" onClick={() => (stage = 'settings')}>Skip</BigButton>
				<BigButton color="green" onClick={importProfiles}>Import</BigButton>
			</div>
		{:else if stage === 'settings'}
			<p>
				Lastly, make sure your settings are correct.
			</p>

			<p class="mt-1">
				You can always edit these later by going to <b>Edit > Settings</b>.
			</p>

			<div class="flex flex-col mt-3 gap-1">
				<ZoomLevelPref />

				<PathSetting label="Steam executable" key="steam_exe_path" type="file">
					Path to the Steam executable.
				</PathSetting>

				<PathSetting label="Download cache directory" key="cache_dir" type="dir">
					Directory where cached mods are stored. 
					<br/>
					Changing this will move the existing cache.
				</PathSetting>
		
				<PathSetting label="Data directory" key="data_dir" type="dir">
					Directory where the profiles, logs and other app data is stored. 
					<br/>
					Changing this will move the existing data.
				</PathSetting>
		
				<PathSetting label="Temp directory" key="temp_dir" type="dir">
					Directory where temporary files are stored, for example import and export files.
				</PathSetting>
			</div>

			<div class="flex mt-3 justify-between">
				<BigButton
					color="gray"
					onClick={() => (stage = importData.r2modman || importData.thunderstore ? 'importProfiles' : 'gameSelect')}>Back</BigButton
				>
				<BigButton color="green" onClick={() => (stage = 'end')}>Next</BigButton>
			</div>
		{:else if stage === 'end'}
			<p>That's it, you're all set up to start modding!</p>

			<p class="mt-1">
				If you have any questions or need help, feel free to ask in the <a
					href="https://discord.gg/lcmod"
					target="_blank"
					class="text-green-400 hover:underline">Lethal Company Modding Discord server</a
				>.
			</p>
		{/if}
	</div>
</Popup>
