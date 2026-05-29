use std::collections::HashMap;

use crate::error::Result;

#[cfg(target_os = "macos")]
pub fn fetch() -> Result<HashMap<String, String>> {
    use std::cell::RefCell;
    use std::ptr::NonNull;
    use std::sync::{Arc, Condvar, Mutex};

    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool, ProtocolObject};
    use objc2::ClassType;
    use objc2_contacts::{
        CNAuthorizationStatus, CNContact, CNContactEmailAddressesKey,
        CNContactFamilyNameKey, CNContactFetchRequest, CNContactGivenNameKey,
        CNContactPhoneNumbersKey, CNContactStore, CNEntityType, CNKeyDescriptor,
    };
    use objc2_foundation::{NSArray, NSError, NSString};

    unsafe {
        let store = CNContactStore::new();

        // Check current authorisation status
        let status = CNContactStore::authorizationStatusForEntityType(CNEntityType::Contacts);

        match status {
            CNAuthorizationStatus::Denied | CNAuthorizationStatus::Restricted => {
                tracing::warn!(
                    "Contacts.app access denied — grant access in \
                     System Settings → Privacy & Security → Contacts, then re-run sync"
                );
                return Ok(HashMap::new());
            }
            CNAuthorizationStatus::NotDetermined => {
                // Request access and block until the user responds
                let granted = Arc::new((Mutex::new(false), Condvar::new()));
                let granted_clone = Arc::clone(&granted);

                let block = block2::RcBlock::new(move |access_granted: Bool, _error: *mut NSError| {
                    let (lock, cvar) = &*granted_clone;
                    *lock.lock().unwrap() = access_granted.as_bool();
                    cvar.notify_one();
                });

                store.requestAccessForEntityType_completionHandler(
                    CNEntityType::Contacts,
                    &block,
                );

                // Wait for the completion handler
                let (lock, cvar) = &*granted;
                let result = lock.lock().unwrap();
                let result = cvar.wait(result).unwrap();
                if !*result {
                    tracing::warn!(
                        "Contacts.app access not granted — \
                         grant access in System Settings → Privacy & Security → Contacts"
                    );
                    return Ok(HashMap::new());
                }
            }
            _ => {} // Authorized — proceed
        }

        fn retain_key(key: &'static NSString) -> Retained<ProtocolObject<dyn CNKeyDescriptor>> {
            // SAFETY: NSString implements CNKeyDescriptor. Memory layout of
            // Retained<AnyObject> and Retained<ProtocolObject<dyn CNKeyDescriptor>>
            // is identical. ProtocolObject::from_retained cannot be used here because
            // ImplementedBy<AnyObject> is not generated for this protocol type.
            unsafe {
                let ptr: *mut AnyObject = key as *const NSString as *mut NSString as *mut AnyObject;
                let retained: Retained<AnyObject> = Retained::retain(ptr).unwrap();
                std::mem::transmute(retained)
            }
        }

        let keys_vec: Vec<Retained<ProtocolObject<dyn CNKeyDescriptor>>> = vec![
            retain_key(CNContactGivenNameKey),
            retain_key(CNContactFamilyNameKey),
            retain_key(CNContactPhoneNumbersKey),
            retain_key(CNContactEmailAddressesKey),
        ];
        let keys_array = NSArray::from_vec(keys_vec);

        let request = CNContactFetchRequest::initWithKeysToFetch(
            CNContactFetchRequest::alloc(),
            &keys_array,
        );

        let entries: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());

        let block = block2::StackBlock::new(
            |contact: NonNull<CNContact>, _stop: NonNull<Bool>| {
                let contact = contact.as_ref();
                let given = contact.givenName().to_string();
                let family = contact.familyName().to_string();
                let name = format!("{given} {family}").trim().to_string();
                if name.is_empty() {
                    return;
                }
                let phones = contact.phoneNumbers();
                for labeled in phones.iter() {
                    let raw = labeled.value().stringValue().to_string();
                    entries.borrow_mut().push((normalize_phone(&raw), name.clone()));
                }
                let emails = contact.emailAddresses();
                for labeled in emails.iter() {
                    let email = labeled.value().to_string().to_lowercase();
                    entries.borrow_mut().push((email, name.clone()));
                }
            },
        );

        let mut error = None;
        let ok = store.enumerateContactsWithFetchRequest_error_usingBlock(
            &request,
            Some(&mut error),
            &block,
        );

        if !ok || error.is_some() {
            tracing::warn!(
                "Contacts enumeration failed — \
                 grant access in System Settings → Privacy & Security → Contacts"
            );
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for (key, value) in entries.into_inner() {
            if let Some(existing) = map.get(&key) {
                if existing != &value {
                    tracing::warn!("Duplicate contact key {key:?}: keeping {existing:?}, ignoring {value:?}");
                }
            } else {
                map.insert(key, value);
            }
        }
        Ok(map)
    }
}

fn normalize_phone(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for (i, c) in raw.chars().enumerate() {
        if c.is_ascii_digit() || (i == 0 && c == '+') {
            result.push(c);
        }
    }
    result
}

#[cfg(not(target_os = "macos"))]
pub fn fetch() -> Result<HashMap<String, String>> {
    Ok(HashMap::new())
}
