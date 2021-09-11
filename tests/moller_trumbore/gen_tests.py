import math
import random
import pyaml


class Vec3:
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

    def sub(self, v):
        return Vec3(self.x - v.x,
                    self.y - v.y,
                    self.z - v.z)

    def dot(self, v):
        return self.x * v.x + self.y * v.y + self.z * v.z

    def cross(self, v):
        return Vec3(self.y * v.z - self.z * v.y,
                    self.z * v.x - self.x * v.z,
                    self.x * v.y - self.y * v.x)

    def length(self):
        return math.sqrt(self.x * self.x +
                         self.y * self.y +
                         self.z * self.z)

    def normalize(self):
        l = self.length()
        return Vec3(self.x / l, self.y / l, self.z / l)


class Ray:
    def __init__(self, orig=None, direction=None):
        self.orig = orig
        self.direction = direction


def ray_triangle_intersect(r, v0, v1, v2):
    v0v1 = v1.sub(v0)
    v0v2 = v2.sub(v0)
    pvec = r.direction.cross(v0v2)

    det = v0v1.dot(pvec)

    if abs(det) < 0.000001:
        return None

    invDet = 1.0 / det
    tvec = r.orig.sub(v0)
    u = tvec.dot(pvec) * invDet

    if u < 0 or u > 1:
        return None

    qvec = tvec.cross(v0v1)
    v = r.direction.dot(qvec) * invDet

    if v < 0 or u + v > 1:
        return None
    dist = v0v2.dot(qvec) * invDet
    if dist < 0:
        return None
    return (dist, u, v)


def random_vertex():
  return Vec3(random.random() * 2.0 - 1.0,
              random.random() * 2.0 - 1.0,
              random.random() * 2.0 - 1.0)


def generate_random_triangles(numTriangles):
    vertices = [None] * numTriangles * 3

    for i in range(numTriangles):
        vertices[i*3 + 0] = random_vertex()
        vertices[i*3 + 1] = random_vertex()
        vertices[i*3 + 2] = random_vertex()

    return vertices


def random_sphere():
    r1 = random.random()
    r2 = random.random()
    lat = math.acos(2*r1 - 1) - math.pi/2
    lon = 2*math.pi * r2

    return Vec3(math.cos(lat) * math.cos(lon),
                math.cos(lat) * math.sin(lon),
                math.sin(lat))


def vec3_to_list(vec):
    return [vec.x, vec.y, vec.z]

NUM_RAYS = 100
NUM_TRIANGLES = 400

random.seed()

vertices = generate_random_triangles(NUM_TRIANGLES)
num_vertices = NUM_TRIANGLES * 3

num_hit = 0
num_miss = 0

r = Ray()

data_hit = []
data_miss = []

for i in range(0, NUM_RAYS):
    r.orig = random_sphere()
    r.direction = random_sphere().sub(r.orig).normalize()

    for j in range(0, NUM_TRIANGLES):
        hit = ray_triangle_intersect(r, vertices[j*3 + 0],
                                      vertices[j*3 + 1],
                                      vertices[j*3 + 2])
        data = { "triangle": {
                  "v0": vec3_to_list(vertices[j*3 + 0]),
                  "v1": vec3_to_list(vertices[j*3 + 1]),
                  "v2": vec3_to_list(vertices[j*3 + 2])
                  },
                "ray": {
                    "position": vec3_to_list(r.orig),
                    "direction": vec3_to_list(r.direction)
                    }}
        if hit:
          num_hit += 1
          data["hit"] = { "dist": hit[0],
                         "u": hit[1],
                         "v": hit[2] }
          data_hit.append(data)
        else:
          num_miss += 1
          data_miss.append(data)

num_tests = NUM_RAYS * NUM_TRIANGLES
hit_perc  = float(num_hit) / num_tests * 100
miss_perc = float(num_miss) / num_tests * 100

with open('hit_tests.yml', 'w') as f:
    print(f"Generate hit_tests.yml: {len(data_hit)} hits")
    pyaml.dump(data_hit, f)

with open('miss_tests.yml', 'w') as f:
    print(f"Generate miss_tests.yml: {len(data_hit)} misses")
    pyaml.dump(data_miss[:len(data_hit)], f)

