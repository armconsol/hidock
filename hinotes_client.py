"""
HiNotes API Client (Unofficial)

A Python client for interacting with the HiNotes API.
Created through reverse engineering - USE AT YOUR OWN RISK.

This may violate HiNotes Terms of Service. Consider contacting
HiDock for official API access before using in production.
"""

import requests
from typing import Optional, Dict, List, Any
from datetime import datetime, timedelta
import json


class HiNotesClient:
    """
    Unofficial HiNotes API Client

    Usage:
        client = HiNotesClient()
        client.authenticate_with_credentials("email@example.com", "password")
        notes = client.list_notes()
    """

    BASE_URL = "https://hinotes.hidock.com/v1"

    def __init__(self, auth_token: Optional[str] = None):
        """
        Initialize the HiNotes client

        Args:
            auth_token: Optional authentication token. If not provided,
                       you'll need to authenticate using one of the auth methods.
        """
        self.session = requests.Session()
        self.auth_token = auth_token

        if auth_token:
            self.session.headers.update({
                'Authorization': f'Bearer {auth_token}'
            })

    def _request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """
        Make an API request

        Args:
            method: HTTP method (GET, POST, etc.)
            endpoint: API endpoint (without /v1 prefix)
            **kwargs: Additional arguments to pass to requests

        Returns:
            JSON response as dictionary

        Raises:
            requests.HTTPError: If the request fails
        """
        url = f"{self.BASE_URL}/{endpoint.lstrip('/')}"

        response = self.session.request(method, url, **kwargs)
        response.raise_for_status()

        return response.json()

    # Authentication Methods

    def authenticate_with_credentials(self, email: str, password: str) -> Dict[str, Any]:
        """
        Authenticate with email and password

        Args:
            email: User email address
            password: User password

        Returns:
            Authentication response with token
        """
        response = self._request('POST', '/user/signin', json={
            'email': email,
            'password': password
        })

        # Extract token from response (exact field name unknown - adjust as needed)
        if 'token' in response:
            self.auth_token = response['token']
            self.session.headers.update({
                'Authorization': f'Bearer {self.auth_token}'
            })

        return response

    def register_user(self, email: str, password: str, name: str) -> Dict[str, Any]:
        """
        Register a new user account

        Args:
            email: Email address
            password: Password
            name: Display name

        Returns:
            Registration response
        """
        return self._request('POST', '/user/register', json={
            'email': email,
            'password': password,
            'name': name
        })

    def logout(self) -> Dict[str, Any]:
        """Logout current user"""
        return self._request('POST', '/user/logout')

    # User Management

    def get_user_info(self) -> Dict[str, Any]:
        """Get current user information"""
        return self._request('POST', '/user/info')

    def update_user_name(self, name: str) -> Dict[str, Any]:
        """
        Update user display name

        Args:
            name: New display name
        """
        return self._request('POST', '/user/rename', json={'name': name})

    # Device Management

    def list_devices(self) -> List[Dict[str, Any]]:
        """List connected HiDoc devices"""
        response = self._request('POST', '/user/device/list')
        return response.get('devices', [])

    def bind_device(self, device_id: str, device_name: str) -> Dict[str, Any]:
        """
        Connect a new HiDoc device

        Args:
            device_id: Device identifier
            device_name: Display name for device
        """
        return self._request('POST', '/user/device/bind', json={
            'deviceId': device_id,
            'deviceName': device_name
        })

    def unbind_device(self, device_id: str) -> Dict[str, Any]:
        """
        Disconnect a HiDoc device

        Args:
            device_id: Device identifier
        """
        return self._request('POST', '/user/device/unbind', json={
            'deviceId': device_id
        })

    def get_device_status(self) -> Dict[str, Any]:
        """Get device connection status"""
        return self._request('GET', '/user/device/status')

    def list_device_files(self) -> List[Dict[str, Any]]:
        """List files on connected device"""
        return self._request('GET', '/user/device/file/list')

    # Notes Management

    def list_notes(
        self,
        folder_id: str = "-1",
        page_index: int = 0,
        page_size: int = 20,
        sort_type: str = "desc",
        sort_field: str = "createtime"
    ) -> Dict[str, Any]:
        """
        List recording notes

        Args:
            folder_id: Folder ID to filter (-1 for all)
            page_index: Page number (0-based)
            page_size: Items per page
            sort_type: Sort direction ('asc' or 'desc')
            sort_field: Field to sort by

        Returns:
            Dictionary with notes list and pagination info
        """
        params = {
            'folderId': folder_id,
            'pageIndex': page_index,
            'pageSize': page_size,
            'sortType': sort_type,
            'sortField': sort_field
        }
        return self._request('GET', '/note/recording/list', params=params)

    def list_whispers(
        self,
        page_size: int = 20,
        sort_field: str = "create_time"
    ) -> Dict[str, Any]:
        """
        List whisper notes (quick voice notes)

        Args:
            page_size: Items per page
            sort_field: Field to sort by
        """
        params = {
            'pageSize': page_size,
            'sortField': sort_field
        }
        return self._request('GET', '/note/whisper/list', params=params)

    def delete_note(self, note_id: str) -> Dict[str, Any]:
        """
        Delete a note

        Args:
            note_id: Note identifier
        """
        return self._request('POST', '/note/delete', json={'noteId': note_id})

    def rate_note(self, note_id: str, rating: int) -> Dict[str, Any]:
        """
        Rate transcription quality

        Args:
            note_id: Note identifier
            rating: Rating from 1-5
        """
        return self._request('POST', '/note/rate', json={
            'noteId': note_id,
            'rating': rating
        })

    # Folder Management

    def list_folders(self) -> List[Dict[str, Any]]:
        """List user folders"""
        response = self._request('POST', '/folder/list')
        return response.get('folders', [])

    def create_folder(self, name: str) -> Dict[str, Any]:
        """
        Create new folder

        Args:
            name: Folder name
        """
        return self._request('POST', '/folder/create', json={'name': name})

    def rename_folder(self, folder_id: str, name: str) -> Dict[str, Any]:
        """
        Rename folder

        Args:
            folder_id: Folder identifier
            name: New folder name
        """
        return self._request('POST', '/folder/rename', json={
            'folderId': folder_id,
            'name': name
        })

    def delete_folder(self, folder_id: str) -> Dict[str, Any]:
        """
        Delete folder

        Args:
            folder_id: Folder identifier
        """
        return self._request('POST', '/folder/remove', json={'folderId': folder_id})

    def assign_note_to_folder(self, note_id: str, folder_id: str) -> Dict[str, Any]:
        """
        Move note to folder

        Args:
            note_id: Note identifier
            folder_id: Target folder identifier
        """
        return self._request('POST', '/folder/assign', json={
            'noteId': note_id,
            'folderId': folder_id
        })

    # To-Do Management

    def list_todos(
        self,
        state: str = "open",
        page_size: int = 10,
        tz_offset: int = 0,
        due_date_start: Optional[datetime] = None,
        due_date_end: Optional[datetime] = None
    ) -> Dict[str, Any]:
        """
        List to-do items

        Args:
            state: Filter by state ('open' or 'closed')
            page_size: Items per page
            tz_offset: Timezone offset in minutes
            due_date_start: Start date filter
            due_date_end: End date filter
        """
        params = {
            'pageSize': page_size,
            'state': state,
            'tzOffset': tz_offset
        }

        if due_date_start:
            params['dueDateStart'] = due_date_start.strftime('%Y-%m-%d %H:%M:%S')
        if due_date_end:
            params['dueDateEnd'] = due_date_end.strftime('%Y-%m-%d %H:%M:%S')

        return self._request('GET', '/todo/list', params=params)

    def update_todo_description(self, todo_id: str, description: str) -> Dict[str, Any]:
        """Update to-do description"""
        return self._request('POST', '/todo/update/description', json={
            'todoId': todo_id,
            'description': description
        })

    def update_todo_due_date(self, todo_id: str, due_date: datetime) -> Dict[str, Any]:
        """Update to-do due date"""
        return self._request('POST', '/todo/update/dueDate', json={
            'todoId': todo_id,
            'dueDate': due_date.strftime('%Y-%m-%d %H:%M:%S')
        })

    def delete_todo(self, todo_id: str) -> Dict[str, Any]:
        """Delete to-do item"""
        return self._request('POST', '/todo/delete', json={'todoId': todo_id})

    # Calendar Management

    def list_calendar_events(
        self,
        start_time: datetime,
        end_time: datetime,
        tz_offset: int = 0
    ) -> Dict[str, Any]:
        """
        List calendar events

        Args:
            start_time: Start of date range
            end_time: End of date range
            tz_offset: Timezone offset in minutes
        """
        params = {
            'start_time': start_time.strftime('%Y-%m-%d %H:%M:%S'),
            'end_time': end_time.strftime('%Y-%m-%d %H:%M:%S'),
            'tz_offset': tz_offset
        }
        return self._request('GET', '/calendar/event/list', params=params)

    def add_calendar_event(
        self,
        title: str,
        start_time: datetime,
        end_time: datetime
    ) -> Dict[str, Any]:
        """Add calendar event"""
        return self._request('POST', '/calendar/event/add', json={
            'title': title,
            'startTime': start_time.strftime('%Y-%m-%d %H:%M:%S'),
            'endTime': end_time.strftime('%Y-%m-%d %H:%M:%S')
        })

    # Settings

    def get_settings(self) -> Dict[str, Any]:
        """Get user settings"""
        return self._request('POST', '/user/setting/get')

    def save_settings(self, settings: Dict[str, Any]) -> Dict[str, Any]:
        """
        Save user settings

        Args:
            settings: Settings dictionary
        """
        return self._request('POST', '/user/setting/save', json=settings)

    # Sync

    def sync_changes(self) -> Dict[str, Any]:
        """Synchronize changes with server"""
        return self._request('POST', '/changes')

    def get_entry_info(self) -> Dict[str, Any]:
        """Get application entry data"""
        return self._request('POST', '/entry/info')


# Example usage
if __name__ == "__main__":
    # Initialize client
    client = HiNotesClient()

    # Authenticate (replace with your credentials)
    # WARNING: This is for demonstration only. Never hardcode credentials.
    # client.authenticate_with_credentials("your-email@example.com", "your-password")

    # Get user info
    # user_info = client.get_user_info()
    # print("User Info:", json.dumps(user_info, indent=2))

    # List notes
    # notes = client.list_notes(page_size=10)
    # print("Notes:", json.dumps(notes, indent=2))

    # List devices
    # devices = client.list_devices()
    # print("Devices:", json.dumps(devices, indent=2))

    # List todos for today
    # from datetime import datetime
    # today_start = datetime.now().replace(hour=0, minute=0, second=0)
    # today_end = datetime.now().replace(hour=23, minute=59, second=59)
    # todos = client.list_todos(due_date_start=today_start, due_date_end=today_end)
    # print("Today's Todos:", json.dumps(todos, indent=2))

    print("HiNotes API Client initialized. See code comments for usage examples.")
    print("WARNING: This is unofficial and may violate Terms of Service.")
    print("Contact HiDock for official API access before production use.")
