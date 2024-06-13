<script lang="ts">
	import { Menubar } from 'bits-ui';
</script>

<Menubar.Item
	class="pl-4 pr-12 py-1 mx-1 text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
	on:click
>
	<slot />
</Menubar.Item>
