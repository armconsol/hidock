import json, sys, os

sys.path.insert(0, "/home/sarman/repos/hinotes/API_Notes")
from hinotes_client import HiNotesClient

HINOTES_TOKEN = os.environ["HINOTES_TOKEN"]
client = HiNotesClient(auth_token=HINOTES_TOKEN)

# Folder map: name -> id
folders = {
    "MSI": "6169242114581471232",
    "Cut-Planning": "6176415505491038208",
    "StandUp": "6176415737050177536",
    "Postmortums": "6176416093297577984",
    "Onboarding": "6176484876359962624",
    "Projects": "6176626741327564800",
    "AI-Tooling": "6179629252871331840",
    "Support": "6181257996291674112",
    "Noise": "6221811993251299328",
}

# Template map
templates = {
    "General Meeting": "1478ae6725524380bc380ae539656543",
    "Client Meeting": "33a21641a49b40ca9702472b2a3ef1de",
    "Project Sync": "2c6dcca1da52405a8826ff9173b43a78",
    "Status Update Meeting": "7007ec2925f445279e848ef9b6a11a9ef",
    "Retrospective Meeting": "3fd3044d4fab46799e638be2e54ed013",
    "Regular Project Meeting": "1f4f3101ec9a4c3b8eb26d35bf643ea9",
    "All Hands Meeting": "41d5d7f540a0401ba6646a886cc67fa4",
}

# Note classifications: (note_id, folder, template_action_or_none)
# NEVER assign to MSI
notes_to_classify = [
    ("6270735392878772224", "Support", "Status Update Meeting"),    # paging mute, FreeSWITCH
    ("6270735933977534464", "Support", "Status Update Meeting"),    # memory leak, FreeSWITCH outage
    ("6270736090089529344", "Support", "Status Update Meeting"),   # voicemail on outage ticket
    ("6270736174697025536", "Support", "Status Update Meeting"),    # alerts, monitoring, FSW
    ("6270737075650306048", "Support", "Client Meeting"),           # ValorCAD tech support call
    ("6270737970060476416", "Support", "Retrospective Meeting"),    # alert review, post-mortem style
    ("6270739174496813056", "AI-Tooling", "Status Update Meeting"), # Uptrace logging infrastructure
    ("6270739558174957568", "Support", "Status Update Meeting"),    # alert noise, FreeSWITCH
    ("6270740345647144960", "Support", "Status Update Meeting"),    # audio quality issues
    ("6270741258596126720", "Projects", "Project Sync"),            # Grand Traverse closure
    ("6267723357161136128", "Projects", "Project Sync"),            # site upgrade planning
    ("6270742356211941376", "Projects", "Project Sync"),            # ISP provisioning, SIP trunk
    ("6267723611436621824", "Projects", "Regular Project Meeting"), # Frappe evaluation as Zoho alt
    ("6267724433016897536", "Projects", "Project Sync"),            # Decatur site transition
    ("6270748149149732864", "Noise", None),                         # pre-meeting setup
    ("6270749532716417024", "Noise", None),                         # Home Depot customer service
    ("6270749676396494848", "Noise", None),                         # Home Depot call connection
    ("6270750632395816960", "Noise", None),                         # raw recording, no template
    ("6270750756295548928", "Noise", None),                         # Hangteszt test recording
    ("6270750876672081920", "Noise", None),                         # no relevant content
    ("6270751007601475584", "Noise", None),                         # audio check session
    ("6270751123179704320", "Noise", None),                         # audio test
    ("6270751263823118336", "Noise", None),                         # audio test
    ("6270751384518402048", "Noise", None),                         # raw recording
    ("6270751511823917056", "Noise", None),                         # raw recording
    ("6270752365901647872", "Noise", None),                         # raw recording
    ("6270752504162684928", "Noise", None),                         # automated voicemail
    ("6270752991758913536", "Noise", None),                         # voicemail system notification
    ("6270753104619257856", "Noise", None),                         # raw recording
    ("6270756708214251520", "Noise", None),                         # raw recording
    ("6270757340446867456", "Noise", None),                         # Connexus credit union personal
    ("6270757754558877696", "Noise", None),                         # Corda credit union personal
    ("6270757931826954240", "Noise", None),                         # Corda IVR personal
    ("6270760709882257408", "Noise", None),                         # raw recording
    ("6267725060451213312", "Noise", None),                         # single utterance Amen
]

results = []
for nid, folder, template_action in notes_to_classify:
    fid = folders[folder]
    
    # Assign to folder
    try:
        client._post_form("/folder/assign", {"noteId": nid, "folderId": fid})
        assign_ok = True
    except Exception as e:
        assign_ok = False
        results.append(f"{nid} -> {folder}: ASSIGN ERROR: {e}")
        continue
    
    # Template change
    tpl_note = client._post_form("/v2/note/info", {"id": nid})
    cur_code = tpl_note.get("aiEngineTemplate", {}).get("templateCode")
    
    if template_action and cur_code == "1478ae6725524380bc380ae539656543":
        tpl_code = templates[template_action]
        try:
            client._post_form("/v2/note/summarize", {
                "noteId": nid,
                "aiEngine": "claude-4.6",
                "templateCode": tpl_code,
                "tzOffset": 300,
            })
            # Verify summary
            verify = client._post_form("/v2/note/info", {"id": nid})
            state = verify.get("state", "unknown")
            results.append(f"{nid} -> {folder} (template: {template_action}, saved: {state})")
        except Exception as e:
            results.append(f"{nid} -> {folder} (template: {template_action}: TEMPLATE ERROR: {e})")
    else:
        results.append(f"{nid} -> {folder} (template: none, cur={cur_code})")

for r in results:
    print(r)
print(f"Total processed: {len(results)}")
