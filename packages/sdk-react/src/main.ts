import type {
	AuthClient,
	AuthCallback,
	UserState,
	User,
	SetPassword,
} from '@riezler/auth-sdk'

import {
	createContext,
	useContext,
	useEffect,
	useRef,
	useState,
	useCallback,
} from 'react'

export let Auth = createContext<AuthClient | null>(null)

export function useAuthStateChange(fn: AuthCallback) {
	let auth = useContext(Auth)
	let cb = useRef(fn)

	useRef(() => {
		cb.current = fn
	})

	useEffect(() => {
		if (auth === null) {
			return
		}

		let sub = auth.authStateChange((user: User) => {
			cb.current(user)
		})

		return () => {
			sub.unsubscribe()
		}
	}, [auth])
}

export function useAuth(): AuthClient {
	let auth = useContext(Auth)

	if (auth === null) {
		throw new AuthClientError('signIn')
	}

	return auth
}

export class AuthClientError extends Error {
	constructor(name: string) {
		super(`Can not use auth of null, did you initialize the auth client?`)
		this.name = 'AuthClientError'
	}
}