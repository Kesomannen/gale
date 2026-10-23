import json
import os

MESSAGES_DIRECTORY = "./messages"
SOURCE_LANGUAGE = "en"

languages = {}

for filename in os.listdir(MESSAGES_DIRECTORY):
    file = os.path.join(MESSAGES_DIRECTORY, filename)
    language = filename.split(".")[0]
    with open(file, "r") as f:
        messages = json.load(f)
    languages[language] = messages

print(f"Read {len(languages)} languages from {MESSAGES_DIRECTORY}.")

source_messages = languages[SOURCE_LANGUAGE]
for language, messages in languages.items():
    if language == SOURCE_LANGUAGE:
        continue

    missing_keys = set(source_messages.keys()) - set(messages.keys())
    if missing_keys:
        print(f"Missing keys in {language}:")
        for key in missing_keys:
            print(f"  {key}")

    extra_keys = set(messages.keys()) - set(source_messages.keys())
    if extra_keys:
        print(f"Extra keys in {language}:")
        for key in extra_keys:
            print(f"  {key}")

print("Message check complete.")
