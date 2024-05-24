<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import PathSetting from './PathPref.svelte';
	import LaunchModePref from './LaunchModePref.svelte';
	import ZoomLevelPref from './ZoomFactorPref.svelte';
	import { Separator } from 'bits-ui';

	export let open = false;
</script>

<Popup title="Settings" bind:open>
	<div class="flex flex-col gap-1 pt-2">
		<LaunchModePref />
		<ZoomLevelPref />

		<Separator.Root class="h-[1px] my-2 bg-gray-500" />

		<PathSetting label="Steam executable" key="steam_exe_path" type="exe">
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
</Popup>
