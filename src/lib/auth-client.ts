import { createAuthClient } from 'better-auth/react';
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { readTextFile, writeTextFile, BaseDirectory, mkdir, remove } from '@tauri-apps/plugin-fs';

// Cookie file path
const COOKIE_FILE_PATH = 'auth-cookie.txt';

// Load cookie from file
const loadCookieFromFile = async (): Promise<string | null> => {
    try {
        const cookie = await readTextFile(COOKIE_FILE_PATH, { baseDir: BaseDirectory.AppConfig });
        return cookie.trim() || null;
    } catch {
        return null;
    }
};

// Save cookie to file
const saveCookieToFile = async (cookie: string): Promise<void> => {
    try {
        // Ensure directory exists
        await mkdir('', { baseDir: BaseDirectory.AppConfig, recursive: true }).catch(() => {
            // Directory might already exist, ignore error
        });

        // Save cookie file
        await writeTextFile(COOKIE_FILE_PATH, cookie, { baseDir: BaseDirectory.AppConfig });
        console.log('Cookie saved successfully to:', COOKIE_FILE_PATH);
    } catch (error) {
        console.error('Failed to save cookie:', error);
        // Provide more detailed error information in development environment
        if (typeof error === 'object' && error !== null) {
            console.error('Error details:', JSON.stringify(error, null, 2));
        }
    }
};

// Delete cookie file
const deleteCookieFile = async (): Promise<void> => {
    try {
        await remove(COOKIE_FILE_PATH, { baseDir: BaseDirectory.AppConfig });
        console.log('Cookie file deleted successfully');
    } catch (error) {
        console.error('Failed to delete cookie file:', error);
        // Ignore error when file doesn't exist
        if (typeof error === 'object' && error !== null) {
            console.error('Error details:', JSON.stringify(error, null, 2));
        }
    }
};

/**
 * Custom fetch implementation that handles Tauri-specific requirements and cookie management
 *
 * Cloud API requires cookies for authentication. For Tauri 2, you can simply replace customFetchImpl and all cloud API calls with Tauri HTTP plugin's fetch
 * For Tauri 1, window.fetch works normally on Windows, but Mac's WKWebView doesn't include cookies when making API calls
 * Therefore, we implemented this based on Tauri 1's fetch to automatically add cookies, usage should be like window.fetch rather than Tauri's fetch
 *
 * @param {RequestInfo | URL} input - The URL or Request object for the request
 * @param {RequestInit} [init] - Optional request initialization options
 * @returns {Promise<Response>} A promise that resolves to the Response object
 * @throws {Error} If the fetch operation fails
 *
 * @description
 * This function provides a custom fetch implementation that:
 * - Converts standard fetch parameters to Tauri-compatible format
 * - Manages cookie storage and retrieval from file
 * - Handles different body types (JSON, FormData, ArrayBuffer, Blob)
 * - Supports streaming responses with proper content type detection
 * - Converts Tauri Response to standard Response object
 */
export const tauriFetchImpl = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const url = typeof input === 'string' ? input : input.toString();

    const headers = new Headers(init?.headers);
    // Add real User-Agent header
    headers.set('User-Agent', navigator.userAgent);

    // Load cookie from file
    const storedCookie = await loadCookieFromFile();
    if (needAppendCookies(url) && storedCookie) {
        headers.set('Cookie', storedCookie);
    }

    if (needDeleteCookies(url)) {
        await deleteCookieFile();
    }

    const response = await tauriFetch(url, {
        ...init,
        method: init?.method?.toUpperCase() ?? 'GET',
        headers,
    });

    // If response contains api/auth/sign-in/, store cookie
    if (needSaveCookies(url)) {
        const setCookieHeader = response.headers.get('set-cookie');
        if (setCookieHeader) {
            // Split on commas that start a new cookie pair, then keep only name=value
            const cookieValues = setCookieHeader
                .split(/,\s*(?=[^;=]+=)/)
                .map((cookie) => cookie.split(';')[0])
                .join('; ');

            // Check if session_token exists and is not empty
            const sessionTokenMatch = cookieValues.match(/session_token=([^;]+)/);
            if (sessionTokenMatch && sessionTokenMatch[1] && sessionTokenMatch[1].trim() !== '') {
                await saveCookieToFile(cookieValues);
            } else {
                console.log('Session token is empty or not found, skipping cookie save');
            }
        }
    }

    return response;
};


const needAppendCookies = (url: string): boolean => {
    return (
        url.includes(import.meta.env.VITE_API_BASE_URL) &&
        !url.includes('api/auth/sign-in/') &&
        !url.includes('api/auth/sign-up/')
    );
};

const needDeleteCookies = (url: string): boolean => {
    return url.includes(import.meta.env.VITE_API_BASE_URL) && url.includes('sign-out');
};

// Tauri v2 webviews run under the `tauri://localhost` origin, which is not a
// valid `http(s)://` base URL for better-auth. When no backend URL is
// configured we fall back to a syntactically valid placeholder so the auth
// client can be constructed (auth-dependent UI will simply fail its network
// calls gracefully) instead of crashing the whole app at module load.
const apiBaseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost';

const needSaveCookies = (url: string): boolean => {
    return url.includes(apiBaseUrl) && !url.includes('api/auth/sign-up/');
};

export const authClient = createAuthClient({
    baseURL: apiBaseUrl,
    fetchOptions: { customFetchImpl: tauriFetchImpl },
});
