
#[repr(C)]
pub struct udi_queue_t
{
    pub next: *mut udi_queue_t,
    pub prev: *mut udi_queue_t,
}

extern "C" {
    /**
     * void udi_enqueue(
     *     udi_queue_t *new_el,
     *     udi_queue_t *old_el)
     * {
     *     new_el->next = old_el->next;
     *     new_el->prev = old_el;
     *     old_el->next->prev = new_el;
     *     old_el->next = new_el;
     * }
     */
    pub fn udi_enqueue(new_el: *mut udi_queue_t, old_el: *mut udi_queue_t);
    /**
     * udi_queue_t *udi_dequeue(
     *     udi_queue_t *element)
     * {
     *     element->next->prev = element->prev;
     *     element->prev->next = element->next;
     *     return element;
     * }
     * */
    pub fn udi_dequeue(element: *mut udi_queue_t) -> *mut udi_queue_t;
}

#[allow(non_snake_case)]
pub unsafe fn UDI_QUEUE_INIT(listhead: *mut udi_queue_t) {
    (*listhead).prev = listhead;
    (*listhead).next = listhead;
}
#[allow(non_snake_case)]
pub unsafe fn UDI_QUEUE_EMPTY(listhead: *mut udi_queue_t)->bool {
    (*listhead).next == listhead
}

#[allow(non_snake_case)]
pub unsafe fn UDI_ENQUEUE_HEAD(listhead: *mut udi_queue_t, element: *mut udi_queue_t) {
    udi_enqueue(element, listhead)
}
#[allow(non_snake_case)]
pub unsafe fn UDI_ENQUEUE_TAIL(listhead: *mut udi_queue_t, element: *mut udi_queue_t) {
    udi_enqueue(element, (*listhead).prev)
}
#[allow(non_snake_case)]
pub unsafe fn UDI_QUEUE_INSERT_AFTER(old_el: *mut udi_queue_t, new_el: *mut udi_queue_t) {
    udi_enqueue(new_el, old_el)
}
#[allow(non_snake_case)]
pub unsafe fn UDI_QUEUE_INSERT_BEFORE(old_el: *mut udi_queue_t, new_el: *mut udi_queue_t) {
    udi_enqueue(new_el, (*old_el).prev)
}

#[allow(non_snake_case)]
pub unsafe fn UDI_DEQUEUE_HEAD(listhead: *mut udi_queue_t) -> *mut udi_queue_t {
    udi_dequeue((*listhead).next)
}
#[allow(non_snake_case)]
pub unsafe fn UDI_DEQUEUE_TAIL(listhead: *mut udi_queue_t) -> *mut udi_queue_t {
    udi_dequeue((*listhead).prev)
}
#[allow(non_snake_case)]
pub unsafe fn UDI_QUEUE_REMOVE(element: *mut udi_queue_t) {
    udi_dequeue(element);
}

#[allow(non_snake_case)]
pub unsafe fn UDI_FIRST_ELEMENT(listhead: *mut udi_queue_t) -> *mut udi_queue_t {
    (*listhead).next
}
#[allow(non_snake_case)]
pub unsafe fn UDI_LAST_ELEMENT(listhead: *mut udi_queue_t) -> *mut udi_queue_t {
    (*listhead).prev
}
#[allow(non_snake_case)]
pub unsafe fn UDI_NEXT_ELEMENT(element: *mut udi_queue_t) -> *mut udi_queue_t {
    (*element).next
}
#[allow(non_snake_case)]
pub unsafe fn UDI_PREV_ELEMENT(element: *mut udi_queue_t) -> *mut udi_queue_t {
    (*element).prev
}
//UDI_QUEUE_FOREACH
//UDI_BASE_STRUCT