<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import PathPref from './PathPref.svelte';
	import LaunchModePref from './LaunchModePref.svelte';
	import ZoomLevelPref from './ZoomFactorPref.svelte';
	import { Separator } from 'bits-ui';
	import TogglePref from './TogglePref.svelte';

	export let open = false;
</script>

<Popup title="Settings" bind:open>
	<div class="flex flex-col gap-1 pt-2">
		<LaunchModePref />
		<ZoomLevelPref />

		<Separator.Root class="h-[1px] my-2 bg-gray-500" />

		<PathPref label="Steam executable" key="steam_exe_path" type="file">
			Path to the Steam executable.
		</PathPref>

		<PathPref label="Data directory" key="data_dir" type="dir">
			Directory where profiles, logs and other app data is stored.
			<br />
			Changing this will move the existing data.
		</PathPref>

		<PathPref label="Temp directory" key="temp_dir" type="dir">
			Directory where temporary files are stored, for example import and export files.
		</PathPref>

		<PathPref label="Download cache directory" key="cache_dir" type="dir">
			Directory where cached mods are stored.
			<br />
			Changing this will move the existing cache.
		</PathPref>

		<TogglePref
			label="Use download cache"
			key="enable_mod_cache"
			disableMessage="This will delete all cached mods. Are you sure?"
		>
			Whether to cache downloaded mods. This speeds up install times and lowers bandwidth usage,
			but can take a considerable amount of disk space.
			<br />
			<b>Warning:</b> Disabling this will delete the existing cache.
		</TogglePref>
	</div>
</Popup>
