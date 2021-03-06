import { useEffect, useState, useCallback } from 'react'

import {
	bosonFamily,
	useBoson,
	useSetBoson,
} from '@biotic-ui/boson'

import { Reload } from 'types/utils'
import { CancelToken, useHttp } from 'data/http'
import { ApiError, getErrorCode } from 'error'

export enum Flags {
	SignIn = 'auth::signin',
	SignUp = 'auth::signup',
	PasswordReset = 'action::password_reset',
	VerifyEmail = 'action::verify_email',
	EmailAndPassword = 'method::email_password',
	AuthenticationLink = 'method::authentication_link',
}

export function isFlag(flag: string | Flags): boolean {
	let index = Object.values(Flags).findIndex(value => {
		return value === flag
	})

	return index !== -1
}

export function getFlagsFromRequest(flags: Array<string>): Array<Flags> {
	let items = flags
		.filter(flag => isFlag(flag))

	return (items as Array<Flags>)
}


type ProjectFlags = {
	items?: Array<Flags>;
	loading: boolean;
	error: null | ApiError;
}

type Response = {
	items: Array<string>
}


let flagsFamily = bosonFamily<[string], ProjectFlags>((id) => {
	return {
		key: `project_flags:${id}`,
		defaultValue: {
			items: undefined,
			loading: true,
			error: null,
		}
	}
})

export function useFlags(project: string): ProjectFlags & Reload {
	let [state, setState] = useBoson(flagsFamily(project))
	let [reload, setReload] = useState<boolean>(false)
	let http = useHttp()

	useEffect(() => {

		setState(state => {
			return {
				...state,
				loading: true,
				error: null,
			}
		})

		let source = CancelToken.source()

		let params = {
			project
		}

		let config = {
			params,
			cancelToken: source.token,
		}

		http
			.get<Response>('project/flags', config)
			.then(res => {
				let flags = getFlagsFromRequest(res.data.items)
				setState(state => {
					return {
						items: flags,
						loading: false,
						error: null,
					}
				})
			})
			.catch(err => {
				setState(state => {
					return {
						...state,
						loading: false,
						error: getErrorCode(err)
					}
				})
			})

		return () => {
			source.cancel()
		}

	}, [project, reload, setState, http])

	return {
		...state,
		reload: () => setReload(r => !r)
	}
}

export function useToggleFlags(project: string) {
	let setState = useSetBoson(flagsFamily(project))
	
	let set = useCallback((flag: Flags) => () => {
		setState(state => {
			let { items = [] } = state

			if (items.includes(flag) && flag === Flags.EmailAndPassword) {
				return {
					...state,
					items: items.filter(f => (
						f !== Flags.EmailAndPassword &&
						f !== Flags.PasswordReset &&
						f !== Flags.VerifyEmail
					))
				}
			}

			if (items.includes(flag)) {
				return {
					...state,
					items: items.filter(f => f !== flag)
				}
			}

			return {
				...state,
				items: items.concat([flag])
			}
		})
	}, [setState])

	return set
}


type UpdateFlags = {
	project: string;
	flags: Array<Flags>;
}

type UpdateFlagsRequest = {
	loading: boolean;
	error: null | ApiError;
}

let updateFamily = bosonFamily<[string], UpdateFlagsRequest>(id => {
	return {
		key: `update_project_flags:${id}`,
		defaultValue: {
			loading: false,
			error: null
		}
	}
})

type UseUpdateFlags = [
	(f: Array<Flags>) => Promise<void>,
	UpdateFlagsRequest,
]

export function useUpdateFlags(project: string): UseUpdateFlags {
	let http = useHttp()
	let [state, setState] = useBoson(updateFamily(project))

	let update = useCallback(async (flags: Array<Flags>) => {

		setState({
			loading: true,
			error: null,
		})

		let payload: UpdateFlags = { project, flags }

		try {
			await http
				.post<void>('project/set_flags', payload)

			setState({
				loading: false,
				error: null,
			})

		} catch (err) {
			setState({
				loading: false,
				error: getErrorCode(err),
			})
		}

	}, [project, http, setState])

	return [update, state]
}