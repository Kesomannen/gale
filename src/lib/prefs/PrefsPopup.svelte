<script lang="ts">
	import Popup from '../components/Popup.svelte';
	import PathSetting from './PathPref.svelte';
	import LaunchModePref from './LaunchModePref.svelte';
	import ZoomLevelPref from './ZoomFactorPref.svelte';
	import { Separator } from 'bits-ui';

	export let open = false;
</script>

<Popup title="Preferences" bind:open>
	<div class="flex flex-col gap-1 pt-2">
		<LaunchModePref />
		<ZoomLevelPref />

		<Separator.Root class="h-0.5 my-2 bg-gray-600" />

		<PathSetting label="Steam executable" key="steam_exe_path" type="exe" />
		<PathSetting label="Download cache directory" key="cache_dir" type="dir" />
		<PathSetting label="Data directory" key="data_dir" type="dir" />
		<PathSetting label="Temp directory" key="temp_dir" type="dir" />
	</div>
</Popup>
