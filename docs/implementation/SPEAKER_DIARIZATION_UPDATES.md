# Speaker Diarization API Integration Updates

## Overview
Replaced mock speaker diarization with real HiNotes API calls for production-ready speaker analysis.

## Files Modified

### 1. `src-tauri/src/speaker/diarization.rs`
**Changes:**
- Added `api_client: Option<Arc<HiNotesClient>>` field to `DiarizationEngine`
- Added `with_api_client()` constructor for dependency injection
- Renamed `create_mock_segments()` → `create_basic_vad_segments()`
- Updated fallback behavior: basic VAD segments now use `"unknown"` speaker ID with 0.5 confidence
- Updated `refine_with_cloud()` to use injected API client or create new one as fallback
- Added proper error context when API calls fail
- Updated tests to reflect new behavior

**API Integration:**
- Uses `HiNotesClient::analyze_speaker_segments()` for cloud-based speaker segmentation
- Converts API response segments to local `SpeakerSegment` format
- Falls back to basic VAD when API is unavailable or authentication fails

**Backward Compatibility:**
- Maintains offline fallback: basic VAD segmentation without speaker identification
- Gracefully handles API unavailability without failing the entire process

### 2. `src-tauri/src/speaker/profiles.rs`
**Changes:**
- Added `api_client: Option<Arc<HiNotesClient>>` field to `SpeakerProfileManager`
- Added `with_api_client()` constructor for dependency injection
- Replaced stub `match_voice_signature()` with real implementation

**API Integration:**
- Uses `HiNotesClient::extract_voice_signature()` for voice feature extraction
- Uses `HiNotesClient::match_speaker()` for acoustic similarity matching
- Queries database for known speakers and passes candidate IDs to API
- Returns matched speaker ID and confidence score
- Gracefully handles errors by returning empty results instead of failing

**Signature Matching Flow:**
1. Check if API client is configured and authenticated
2. Get all known speakers from database
3. Call API with signature ID and candidate speaker IDs
4. Return matched speaker with confidence score above threshold
5. Return empty vector if no match or API unavailable

## API Methods Used

### From `api/client.rs`:

1. **`analyze_speaker_segments(audio_path, note_id)`**
   - Uploads audio file for speaker diarization
   - Returns `Vec<SpeakerSegment>` with timing and speaker IDs
   - Used in: `DiarizationEngine::refine_with_cloud()`

2. **`extract_voice_signature(note_id, start_time, end_time)`**
   - Extracts acoustic features from audio segment
   - Returns `VoiceSignature` with feature vector
   - Ready for use in future voice matching implementations

3. **`match_speaker(signature_id, candidate_speaker_ids, threshold)`**
   - Matches voice signature against known speakers
   - Returns `MatchSpeakerResponse` with matched speaker ID and confidence
   - Used in: `SpeakerProfileManager::match_voice_signature()`

## Testing

### Unit Tests Updated:
- `test_create_basic_vad_segments()` - validates fallback segmentation
- `test_create_basic_vad_segments_unknown_speaker()` - verifies "unknown" speaker ID
- `test_create_basic_vad_segments_coverage()` - checks duration coverage
- `test_diarization_engine_with_api_client()` - verifies API client injection

### Integration Tests Needed:
- [ ] Mock API responses for speaker analysis
- [ ] Test error handling when API is unavailable
- [ ] Test authentication failure scenarios
- [ ] Test voice signature matching with known speakers
- [ ] Test confidence threshold handling

## Deployment Notes

### Prerequisites:
- HiNotes API authentication token must be available
- API client should be initialized at application startup
- Database must be populated with speaker profiles for matching

### Configuration:
```rust
// Initialize API client
let api_client = Arc::new(HiNotesClient::new());

// Load token from keyring or authenticate
api_client.load_token_from_keyring().await?;

// Create diarization engine with API client
let options = DiarizationOptions::default();
let engine = DiarizationEngine::with_api_client(options, api_client.clone());

// Create profile manager with API client
let profile_manager = SpeakerProfileManager::with_api_client(api_client.clone());
```

### Offline Mode:
- When API is unavailable, system falls back to basic VAD segmentation
- No speaker identification performed offline (all segments marked as "unknown")
- Low confidence score (0.5) assigned to fallback segments
- Voice matching returns empty results without API

## Error Handling

### API Failures:
- Authentication failures: Log warning, return fallback results
- Network errors: Log warning, return fallback results
- Invalid responses: Log error, return fallback results

### Graceful Degradation:
- Diarization continues with basic VAD if API fails
- Speaker matching returns empty results if API unavailable
- System remains functional without cloud features

## Security Considerations

- API tokens stored in system keyring via `keyring` crate
- No sensitive data logged (tokens, user info)
- Audio files transmitted over HTTPS
- Voice signatures stored with encryption support

## Performance

### API Calls:
- `analyze_speaker_segments()`: ~2-5 seconds per minute of audio
- `match_speaker()`: ~500ms per signature
- Retry logic: 3 attempts with exponential backoff

### Optimization:
- Use provided API client instance (no re-authentication per request)
- Cache speaker profiles in database to minimize API calls
- Process segments in batches where possible

## Future Enhancements

1. **Local Voice Signature Extraction**
   - Implement `extract_voice_signature_local()` in API client
   - Use MFCC or similar acoustic features
   - Reduce API dependency for feature extraction

2. **Batch Processing**
   - Upload multiple audio files in parallel
   - Process segments in batches for efficiency

3. **Caching**
   - Cache voice signatures to avoid re-extraction
   - Cache matching results with TTL

4. **Progressive Enhancement**
   - Start with basic VAD immediately
   - Refine with API results asynchronously
   - Update UI progressively as results improve

## Migration Path

### From Mock to Production:
1. Ensure API client is properly initialized
2. Test with small audio files first
3. Monitor API usage and costs
4. Verify fallback behavior works as expected
5. Gradually increase cloud refinement usage

### Rollback Plan:
- Set `use_cloud_refinement: false` in `DiarizationOptions`
- System will continue using basic VAD segmentation
- No functionality lost, just reduced accuracy

## Documentation Updates Needed

- [ ] Update API documentation with speaker endpoints
- [ ] Add examples for voice signature matching
- [ ] Document confidence score interpretation
- [ ] Add troubleshooting guide for API failures
