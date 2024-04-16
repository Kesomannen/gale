<script lang="ts">
	import { Tabs } from 'bits-ui';
	import Popup from '../Popup.svelte';
	import PathSetting from './PathSetting.svelte';

	export let open = false;
</script>

<Popup title="Preferences" bind:open>
	<div class="flex flex-col gap-1">
		<PathSetting label="Steam executable" name="steam_exe_path" type="exe" />
		<PathSetting label="Download cache directory" name="cache_path" type="dir" />
		<PathSetting label="Data directory" name="data_path" type="dir" />
	</div>
</Popup>
