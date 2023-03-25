import type { Component } from 'solid-js'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { createSignal, For, onCleanup } from 'solid-js'

import {
	Input,
	InputGroup,
	InputLeftAddon,
	Switch,
	VStack,
	FormLabel,
	Td,
	Table,
	Thead,
	Th,
	TableCaption,
	Tbody,
	Tr,
	Tag,
} from '@hope-ui/solid'

enum Severity {
	Info = 'Info',
	Success = 'Success',
	Error = 'Error',
}

function severityToColor(severity: Severity): 'info' | 'success' | 'danger' {
	switch (severity) {
		case Severity.Info:
			return 'info'
		case Severity.Success:
			return 'success'
		case Severity.Error:
			return 'danger'
	}
}
interface LogEntry {
	severity: Severity
	event: string
	filename?: string
}

const App: Component = () => {
	const [apiKey, setApiKey] = createSignal('')
	const [xmlPath, setXmlPath] = createSignal('')
	const [isRunning, setIsRunning] = createSignal(false)
	const [log, setLog] = createSignal<LogEntry[]>([])
	const addLogEntry = (entry: LogEntry) => {
		setLog([entry, ...log()])
	}
	const isDisabled = () => apiKey() === '' || xmlPath() === ''

	listen<string>('event-log', (event) => {
		addLogEntry(event.payload)
	}).then((unlisten) => {
		onCleanup(async () => {
			await unlisten()
		})
	})

	async function start() {
		try {
			await invoke('watch')
			setIsRunning(true)
			addLogEntry({
				severity: Severity.Info,
				event: 'Watching for new XML files...',
				filename: xmlPath(),
			})
		} catch (err) {
			addLogEntry({ severity: Severity.Error, event: err })
		}
	}
	async function stop() {
		try {
			await invoke('unwatch')
			addLogEntry({
				severity: Severity.Info,
				event: 'Watcher stopped',
				filename: xmlPath(),
			})
		} catch (err) {
			addLogEntry({ severity: Severity.Error, event: err })
		}
		setIsRunning(false)
	}
	async function handleApiKeyChange() {
		if (isRunning()) {
			await stop()
		}
		await invoke('set_api_key', { newApiKey: apiKey() })
	}
	async function changeXmlPath() {
		invoke('pick_folder').then(async (path) => {
			if (path !== xmlPath() && isRunning()) {
				await stop()
			}
			setXmlPath(path)
		})
	}

	return (
		<div id='App'>
			<VStack spacing='0.5em' class='inputs'>
				<InputGroup>
					<InputLeftAddon w={'6em'}>API key</InputLeftAddon>
					<Input
						value={apiKey()}
						onInput={(e) => setApiKey(e.target.value)}
						onChange={(e) => handleApiKeyChange(e.target.value)}
						placeholder='API key from OResults.eu'
					/>
				</InputGroup>
				<InputGroup>
					<InputLeftAddon w={'6em'}>XML files</InputLeftAddon>
					<Input
						value={xmlPath()}
						placeholder='select folder for exported XML files'
						readOnly
						onClick={() => changeXmlPath()}
					/>
				</InputGroup>
			</VStack>
			<VStack class='on-off-toggle'>
				<FormLabel for='switch'>
					{!isRunning() ? 'Stopped' : 'Running'}
				</FormLabel>
				<Switch
					id='switch'
					checked={isRunning()}
					onChange={() => (isRunning() ? stop() : start())}
					disabled={isDisabled()}
					colorScheme='success'
					size='lg'
				/>
			</VStack>
			<main class='log'>
				<Table dense>
					<TableCaption>Log</TableCaption>
					<Thead>
						<Tr>
							<Th>Severity</Th>
							<Th>Event</Th>
							<Th>File</Th>
						</Tr>
					</Thead>
					<Tbody>
						<For each={log()}>
							{(e) => {
								return (
									<Tr>
										<Td>
											<Tag
												colorScheme={severityToColor(
													e.severity
												)}
											>
												{e.severity}
											</Tag>
										</Td>
										<Td>{e.event}</Td>
										<Td>{e.filename}</Td>
									</Tr>
								)
							}}
						</For>
					</Tbody>
				</Table>
			</main>
		</div>
	)
}

export default App
