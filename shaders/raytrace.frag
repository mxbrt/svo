#version 450
#define STACK_SIZE 23
#define PI 3.1415926535897932384626433832795
#define EPS 1e-4

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Globals {
    mat4 camera_rotation;
    vec4 camera_position;
    uint width;
    uint height;
    float aspect_ratio;
    float fov;
};

layout(set = 0, binding = 1) readonly buffer SvoBuffer {
    uint svo_data[];
};

struct StackItem { uint node; float t_max; } stack[STACK_SIZE];
bool raymarch(vec3 o, vec3 d, out float o_t, out vec3 o_color, out vec3 o_normal) {
    o += 1;
    d.x = abs(d.x) > EPS ? d.x : (d.x >= 0 ? EPS : -EPS);
    d.y = abs(d.y) > EPS ? d.y : (d.y >= 0 ? EPS : -EPS);
    d.z = abs(d.z) > EPS ? d.z : (d.z >= 0 ? EPS : -EPS);

    // Precompute the coefficients of tx(x), ty(y), and tz(z).
    vec3 t_coef = 1.0f / -abs(d);
    vec3 t_bias = t_coef * o;

    uint oct_mask = 7u;
    if(d.x > 0.0f) oct_mask ^= 1u, t_bias.x = 3.0f * t_coef.x - t_bias.x;
    if(d.y > 0.0f) oct_mask ^= 2u, t_bias.y = 3.0f * t_coef.y - t_bias.y;
    if(d.z > 0.0f) oct_mask ^= 4u, t_bias.z = 3.0f * t_coef.z - t_bias.z;

    // Initialize the active span of t-values.
    float t_min = max(max(2.0f * t_coef.x - t_bias.x, 2.0f * t_coef.y - t_bias.y), 2.0f * t_coef.z - t_bias.z);
    float t_max = min(min(       t_coef.x - t_bias.x,        t_coef.y - t_bias.y),        t_coef.z - t_bias.z);
    t_min = max(t_min, 0.0f);
    float h = t_max;

    uint parent_idx = 0u;
    uint cur    = 0u;
    uint cur_node = svo_data[0];
    vec3 pos    = vec3(1.0f);
    uint idx    = 0u;
    uint  scale      = STACK_SIZE - 1;
    float scale_exp2 = 0.5f; //exp2( scale - STACK_SIZE )
    uint color = 0;

    if(1.5f * t_coef.x - t_bias.x > t_min) idx ^= 1u, pos.x = 1.5f;
    if(1.5f * t_coef.y - t_bias.y > t_min) idx ^= 2u, pos.y = 1.5f;
    if(1.5f * t_coef.z - t_bias.z > t_min) idx ^= 4u, pos.z = 1.5f;


    while( scale < STACK_SIZE )
    {
        if(cur == 0u) {
            cur_node = svo_data[parent_idx];
        }
        // Determine maximum t-value of the cube by evaluating
        // tx(), ty(), and tz() at its corner.

        vec3 t_corner = pos * t_coef - t_bias;
        float tc_max = min(min(t_corner.x, t_corner.y), t_corner.z);

        // Process voxel if it exists and the active t-span is non-empty.
        uint child_idx = (cur_node & 0x3fffffff) * 2 + (idx ^ oct_mask) * 2;
        uint child = svo_data[child_idx];

        if((child & 0x80000000u) == 0 && t_min <= t_max )
        {
            // INTERSECT
            float tv_max = min(t_max, tc_max);
            float half_scale_exp2 = scale_exp2 * 0.5f;
            vec3 t_center = half_scale_exp2 * t_coef + t_corner;

            if( t_min <= tv_max )
            {
                // leaf node
                if( (child & 0x40000000u) != 0 ) {
                    color = svo_data[child_idx + 1];
                    break;
                }

                // PUSH
                if (tc_max < h) {
                    stack[ scale ].node = parent_idx;
                    stack[ scale ].t_max = t_max;
                }
                h = tc_max;
                parent_idx = child_idx;
                idx = 0u;
                scale -= 1;
                scale_exp2 = half_scale_exp2;

                if(t_center.x > t_min) idx ^= 1u, pos.x += scale_exp2;
                if(t_center.y > t_min) idx ^= 2u, pos.y += scale_exp2;
                if(t_center.z > t_min) idx ^= 4u, pos.z += scale_exp2;

                cur = 0;
                t_max = tv_max;

                continue;
            }
        }

        //ADVANCE
        // Step along the ray
        uint step_mask = 0u;
        if(t_corner.x <= tc_max) step_mask ^= 1u, pos.x -= scale_exp2;
        if(t_corner.y <= tc_max) step_mask ^= 2u, pos.y -= scale_exp2;
        if(t_corner.z <= tc_max) step_mask ^= 4u, pos.z -= scale_exp2;

        // Update active t-span and flip bits of the child slot index.
        t_min = tc_max;
        idx ^= step_mask;

        // Proceed with pop if the bit flips disagree with the ray direction.
        if( (idx & step_mask) != 0 )
        {
            // POP
            // Find the highest differing bit between the two positions.
            uint differing_bits = 0;
            if ((step_mask & 1u) != 0) differing_bits |= floatBitsToUint(pos.x) ^ floatBitsToUint(pos.x + scale_exp2);
            if ((step_mask & 2u) != 0) differing_bits |= floatBitsToUint(pos.y) ^ floatBitsToUint(pos.y + scale_exp2);
            if ((step_mask & 4u) != 0) differing_bits |= floatBitsToUint(pos.z) ^ floatBitsToUint(pos.z + scale_exp2);
            scale = findMSB(differing_bits);
            scale_exp2 = uintBitsToFloat((scale - STACK_SIZE + 127u) << 23u); // exp2f(scale - s_max)

            // Restore parent voxel from the stack.
            parent_idx = stack[scale].node;
            t_max  = stack[scale].t_max;

            // Round cube position and extract child slot index.
            uint shx = floatBitsToUint(pos.x) >> scale;
            uint shy = floatBitsToUint(pos.y) >> scale;
            uint shz = floatBitsToUint(pos.z) >> scale;
            pos.x = uintBitsToFloat(shx << scale);
            pos.y = uintBitsToFloat(shy << scale);
            pos.z = uintBitsToFloat(shz << scale);
            idx  = (shx & 1u) | ((shy & 1u) << 1u) | ((shz & 1u) << 2u);

            // Prevent same parent from being stored again and invalidate cached child descriptor.
            h = 0.0;
            cur = 0;
        }
    }

    vec3 norm, t_corner = t_coef * (pos + scale_exp2) - t_bias;
    if(t_corner.x > t_corner.y && t_corner.x > t_corner.z)
        norm = vec3(-1, 0, 0);
    else if(t_corner.y > t_corner.z)
        norm = vec3(0, -1, 0);
    else
        norm = vec3(0, 0, -1);
    if ((oct_mask & 1u) == 0u) norm.x = -norm.x;
    if ((oct_mask & 2u) == 0u) norm.y = -norm.y;
    if ((oct_mask & 4u) == 0u) norm.z = -norm.z;

    o_normal = norm;
    o_color = vec3( color & 0xffu, (color >> 8u) & 0xffu, (color >> 16u) & 0xffu) * 0.00392156862745098f; // (...) / 255.0f
    o_t = t_min;

    return scale < STACK_SIZE && t_min <= t_max;
}

vec3 shade(vec3 color, vec3 normal, vec3 hit_point) {
    vec3 light_dir = normalize(vec3(-1.0,-1.0,-1.0));
    float shadow_bias = 0.00001;
    vec3 result;

    vec3 shadow_ray_dir = -light_dir;
    float t;
    float albedo = 1.0;
    float light_intensity = 4.0;
    float light_reflected = (albedo / PI) * max(0.0, dot(normal, -light_dir)) * light_intensity;
    return color * light_reflected;
}

bool loop(vec3 o, vec3 d, out float o_t, out vec3 o_color, out vec3 o_normal) {
    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < 10; j++) {
            vec3 o2 = vec3(o.x + i, o.y, o.z + j);
            bool hit = raymarch(o2, d, o_t, o_color, o_normal);
            if (hit) {
                return true;
            }
        }
    }
    return false;
}

void main() {
    vec2 uv = (gl_FragCoord.xy + 0.5) / vec2(width,height);
    float x = (2.0 * uv.x - 1.0) * aspect_ratio * fov;
    float y = (1.0 - 2.0 * uv.y) * fov;
    vec3 dir = normalize((camera_rotation * vec4(x,y,1.0,1.0)).xyz);

    float t;
    vec3 color;
    vec3 normal;
    bool hit = loop(camera_position.xyz, dir, t, color, normal);
    if (hit) {
        vec3 hit_point = (camera_position.xyz + t * dir);
        outColor = vec4(shade(color, normal, hit_point),1.0);
    } else {
        outColor = vec4(1.0);
    }
}